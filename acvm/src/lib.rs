#![warn(unused_crate_dependencies)]
#![warn(unreachable_pub)]

// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub mod compiler;
pub mod pwg;

use crate::pwg::{arithmetic::ArithmeticSolver, oracle::OracleSolver};
use acir::{
    circuit::{
        opcodes::{BlackBoxFuncCall, OracleData},
        Circuit, Opcode,
    },
    native_types::{Expression, Witness},
    BlackBoxFunc,
};
use pwg::{block::Blocks, directives::solve_directives};
use std::collections::BTreeMap;
use thiserror::Error;

// re-export acir
pub use acir;
pub use acir::FieldElement;

// This enum represents the different cases in which an
// opcode can be unsolvable.
// The most common being that one of its input has not been
// assigned a value.
//
// TODO: ExpressionHasTooManyUnknowns is specific for arithmetic expressions
// TODO: we could have a error enum for arithmetic failure cases in that module
// TODO that can be converted into an OpcodeNotSolvable or OpcodeResolutionError enum
#[derive(PartialEq, Eq, Debug, Error)]
pub enum OpcodeNotSolvable {
    #[error("missing assignment for witness index {0}")]
    MissingAssignment(u32),
    #[error("expression has too many unknowns {0}")]
    ExpressionHasTooManyUnknowns(Expression),
}

#[derive(PartialEq, Eq, Debug, Error)]
pub enum OpcodeResolutionError {
    #[error("cannot solve opcode: {0}")]
    OpcodeNotSolvable(#[from] OpcodeNotSolvable),
    #[error("backend does not currently support the {0} opcode. ACVM does not currently have a fallback for this opcode.")]
    UnsupportedBlackBoxFunc(BlackBoxFunc),
    #[error("could not satisfy all constraints")]
    UnsatisfiedConstrain,
    #[error("unexpected opcode, expected {0}, but got {1}")]
    UnexpectedOpcode(&'static str, BlackBoxFunc),
    #[error("expected {0} inputs for function {1}, but got {2}")]
    IncorrectNumFunctionArguments(usize, BlackBoxFunc, usize),
}

#[derive(Debug, PartialEq)]
pub enum OpcodeResolution {
    /// The opcode is resolved
    Solved,
    /// The opcode is not solvable             
    Stalled(OpcodeNotSolvable),
    /// The opcode is not solvable but could resolved some witness
    InProgress,
}

/// Check if all of the inputs to the function have assignments
///
/// Returns the first missing assignment if any are missing
fn first_missing_assignment(
    witness_assignments: &BTreeMap<Witness, FieldElement>,
    func_call: &BlackBoxFuncCall,
) -> Option<Witness> {
    func_call.inputs.iter().find_map(|input| {
        if witness_assignments.contains_key(&input.witness) {
            None
        } else {
            Some(input.witness)
        }
    })
}

pub trait Backend: SmartContract + ProofSystemCompiler + PartialWitnessGenerator {}

/// This component will generate the backend specific output for
/// each OPCODE.
/// Returns an Error if the backend does not support that OPCODE
pub trait PartialWitnessGenerator {
    fn solve(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        blocks: &mut Blocks,
        mut opcode_to_solve: Vec<Opcode>,
    ) -> Result<(Vec<Opcode>, Vec<OracleData>), OpcodeResolutionError> {
        let mut unresolved_opcodes: Vec<Opcode> = Vec::new();
        let mut unresolved_oracles: Vec<OracleData> = Vec::new();
        while !opcode_to_solve.is_empty() || !unresolved_oracles.is_empty() {
            unresolved_opcodes.clear();
            let mut stalled = true;
            let mut opcode_not_solvable = None;
            for opcode in &opcode_to_solve {
                let mut solved_oracle_data = None;
                let resolution = match opcode {
                    Opcode::Arithmetic(expr) => ArithmeticSolver::solve(initial_witness, expr),
                    Opcode::BlackBoxFuncCall(bb_func) => {
                        if let Some(unassigned_witness) =
                            first_missing_assignment(initial_witness, bb_func)
                        {
                            Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::MissingAssignment(
                                unassigned_witness.0,
                            )))
                        } else {
                            self.solve_black_box_function_call(initial_witness, bb_func)
                        }
                    }
                    Opcode::Directive(directive) => solve_directives(initial_witness, directive),
                    Opcode::Block(block) | Opcode::ROM(block) | Opcode::RAM(block) => {
                        blocks.solve(block.id, &block.trace, initial_witness)
                    }
                    Opcode::Oracle(data) => {
                        let mut data_clone = data.clone();
                        let result = OracleSolver::solve(initial_witness, &mut data_clone)?;
                        solved_oracle_data = Some(data_clone);
                        Ok(result)
                    }
                };
                match resolution {
                    Ok(OpcodeResolution::Solved) => {
                        stalled = false;
                    }
                    Ok(OpcodeResolution::InProgress) => {
                        stalled = false;
                        // InProgress Oracles must be externally re-solved
                        if let Some(oracle) = solved_oracle_data {
                            unresolved_oracles.push(oracle);
                        } else {
                            unresolved_opcodes.push(opcode.clone());
                        }
                    }
                    Ok(OpcodeResolution::Stalled(not_solvable)) => {
                        if opcode_not_solvable.is_none() {
                            // we keep track of the first unsolvable opcode
                            opcode_not_solvable = Some(not_solvable);
                        }
                        // We push those opcodes not solvable to the back as
                        // it could be because the opcodes are out of order, i.e. this assignment
                        // relies on a later opcodes' results
                        unresolved_opcodes.push(match solved_oracle_data {
                            Some(oracle_data) => Opcode::Oracle(oracle_data),
                            None => opcode.clone(),
                        });
                    }
                    Err(OpcodeResolutionError::OpcodeNotSolvable(_)) => {
                        unreachable!("ICE - Result should have been converted to GateResolution")
                    }
                    Err(err) => return Err(err),
                }
            }
            // We have oracles that must be externally resolved
            if !unresolved_oracles.is_empty() {
                return Ok((unresolved_opcodes, unresolved_oracles));
            }
            // We are stalled because of an opcode being bad
            if stalled && !unresolved_opcodes.is_empty() {
                return Err(OpcodeResolutionError::OpcodeNotSolvable(
                    opcode_not_solvable
                        .expect("infallible: cannot be stalled and None at the same time"),
                ));
            }
            std::mem::swap(&mut opcode_to_solve, &mut unresolved_opcodes);
        }
        Ok((Vec::new(), Vec::new()))
    }

    fn solve_black_box_function_call(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<OpcodeResolution, OpcodeResolutionError>;
}

pub trait SmartContract {
    // TODO: Allow a backend to support multiple smart contract platforms

    /// Returns an Ethereum smart contract to verify proofs against a given verification key.
    fn eth_contract_from_vk(&self, verification_key: &[u8]) -> String;
}

pub trait ProofSystemCompiler {
    /// The NPC language that this proof system directly accepts.
    /// It is possible for ACVM to transpile to different languages, however it is advised to create a new backend
    /// as this in most cases will be inefficient. For this reason, we want to throw a hard error
    /// if the language and proof system does not line up.
    fn np_language(&self) -> Language;

    // Returns true if the backend supports the selected black box function
    fn black_box_function_supported(&self, opcode: &BlackBoxFunc) -> bool;

    /// Returns the number of gates in a circuit
    fn get_exact_circuit_size(&self, circuit: &Circuit) -> u32;

    /// Generates a proving and verification key given the circuit description
    /// These keys can then be used to construct a proof and for its verification
    fn preprocess(&self, circuit: &Circuit) -> (Vec<u8>, Vec<u8>);

    /// Creates a Proof given the circuit description, the initial witness values, and the proving key
    /// It is important to note that the intermediate witnesses for black box functions will not generated
    /// This is the responsibility of the proof system.
    fn prove_with_pk(
        &self,
        circuit: &Circuit,
        witness_values: BTreeMap<Witness, FieldElement>,
        proving_key: &[u8],
    ) -> Vec<u8>;

    /// Verifies a Proof, given the circuit description, the circuit's public inputs, and the verification key
    fn verify_with_vk(
        &self,
        proof: &[u8],
        public_inputs: BTreeMap<Witness, FieldElement>,
        circuit: &Circuit,
        verification_key: &[u8],
    ) -> bool;
}

/// Supported NP complete languages
/// This might need to be in ACIR instead
#[derive(Debug, Clone)]
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}

pub fn hash_constraint_system(cs: &Circuit) -> [u8; 32] {
    let mut bytes = Vec::new();
    cs.write(&mut bytes).expect("could not serialize circuit");

    use sha2::{digest::FixedOutput, Digest, Sha256};
    let mut hasher = Sha256::new();

    hasher.update(bytes);
    hasher.finalize_fixed().into()
}

pub fn checksum_constraint_system(cs: &Circuit) -> u32 {
    let mut bytes = Vec::new();
    cs.write(&mut bytes).expect("could not serialize circuit");

    use crc32fast::Hasher;
    let mut hasher = Hasher::new();

    hasher.update(&bytes);
    hasher.finalize()
}

#[deprecated(
    note = "For backwards compatibility, this method allows you to derive _sensible_ defaults for black box function support based on the np language. \n Backends should simply specify what they support."
)]
// This is set to match the previous functionality that we had
// Where we could deduce what opcodes were supported
// by knowing the np complete language
pub fn default_is_opcode_supported(
    language: Language,
) -> compiler::transformers::IsOpcodeSupported {
    // R1CS does not support any of the opcode except Arithmetic by default.
    // The compiler will replace those that it can -- ie range, xor, and
    fn r1cs_is_supported(opcode: &Opcode) -> bool {
        matches!(opcode, Opcode::Arithmetic(_))
    }

    // PLONK supports most of the opcodes by default
    // The ones which are not supported, the acvm compiler will
    // attempt to transform into supported gates. If these are also not available
    // then a compiler error will be emitted.
    fn plonk_is_supported(opcode: &Opcode) -> bool {
        !matches!(
            opcode,
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall { name: BlackBoxFunc::AES, .. })
                | Opcode::Block(_)
        )
    }

    match language {
        Language::R1CS => r1cs_is_supported,
        Language::PLONKCSat { .. } => plonk_is_supported,
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acir::{
        circuit::{
            directives::Directive,
            opcodes::{BlackBoxFuncCall, OracleData},
            Opcode,
        },
        native_types::{Expression, Witness},
        FieldElement,
    };

    use crate::{
        pwg::block::Blocks, Backend, OpcodeResolution, OpcodeResolutionError,
        PartialWitnessGenerator,
    };

    struct StubbedPwg;

    impl PartialWitnessGenerator for StubbedPwg {
        fn solve_black_box_function_call(
            &self,
            _initial_witness: &mut BTreeMap<Witness, FieldElement>,
            _func_call: &BlackBoxFuncCall,
        ) -> Result<OpcodeResolution, OpcodeResolutionError> {
            panic!("Path not trodden by this test")
        }
    }

    #[test]
    fn inversion_oracle_equivalence() {
        // Opcodes below describe the following:
        // fn main(x : Field, y : pub Field) {
        //     let z = x + y;
        //     constrain 1/z == Oracle("inverse", x + y);
        // }
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);
        let w_y = Witness(2);
        let w_oracle = Witness(3);
        let w_z = Witness(4);
        let w_z_inverse = Witness(5);
        let opcodes = vec![
            Opcode::Oracle(OracleData {
                name: "invert".into(),
                inputs: vec![Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                    q_c: fe_0,
                }],
                input_values: vec![],
                outputs: vec![w_oracle],
                output_values: vec![],
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
                q_c: fe_0,
            }),
            Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![(fe_1, w_z, w_z_inverse)],
                linear_combinations: vec![],
                q_c: -fe_1,
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
                q_c: fe_0,
            }),
        ];

        let pwg = StubbedPwg;

        let mut witness_assignments = BTreeMap::from([
            (Witness(1), FieldElement::from(2u128)),
            (Witness(2), FieldElement::from(3u128)),
        ]);
        let mut blocks = Blocks::default();
        let (unsolved_opcodes, mut unresolved_oracles) = pwg
            .solve(&mut witness_assignments, &mut blocks, opcodes)
            .expect("should stall on oracle");
        assert!(unsolved_opcodes.is_empty(), "oracle should be removed");
        assert_eq!(unresolved_oracles.len(), 1, "should have an oracle request");
        let mut oracle_data = unresolved_oracles.remove(0);
        assert_eq!(oracle_data.input_values.len(), 1, "Should have solved a single input");

        // Filling data request and continue solving
        oracle_data.output_values = vec![oracle_data.input_values.last().unwrap().inverse()];
        let mut next_opcodes_for_solving = vec![Opcode::Oracle(oracle_data)];
        next_opcodes_for_solving.extend_from_slice(&unsolved_opcodes[..]);
        let (unsolved_opcodes, unresolved_oracles) = pwg
            .solve(&mut witness_assignments, &mut blocks, next_opcodes_for_solving)
            .expect("should be solvable");
        assert!(unsolved_opcodes.is_empty(), "should be fully solved");
        assert!(unresolved_oracles.is_empty(), "should have no unresolved oracles");
    }

    #[test]
    fn test_backend_object_safety() {
        // This test doesn't do anything at runtime.
        // We just want to ensure that the `Backend` trait is object safe and this test will refuse to compile
        // if this property is broken.
        #[allow(dead_code)]
        fn check_object_safety(_backend: Box<dyn Backend>) {}
    }
}
