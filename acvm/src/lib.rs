#![warn(unused_crate_dependencies)]
#![warn(unreachable_pub)]

// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub mod compiler;
pub mod pwg;

use crate::pwg::arithmetic::ArithmeticSolver;
use acir::{
    circuit::{
        directives::{Directive, SolvedLog},
        opcodes::BlackBoxFuncCall,
        Circuit, Opcode,
    },
    native_types::{Expression, Witness},
    BlackBoxFunc,
};
use pwg::block::Blocks;
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

pub trait Backend: SmartContract + ProofSystemCompiler + PartialWitnessGenerator {}

/// This component will generate the backend specific output for
/// each OPCODE.
/// Returns an Error if the backend does not support that OPCODE
pub trait PartialWitnessGenerator {
    fn solve(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        mut opcode_to_solve: Vec<Opcode>,
        logs: &mut Vec<SolvedLog>,
    ) -> Result<(), OpcodeResolutionError> {
        let mut unresolved_opcodes: Vec<Opcode> = Vec::new();
        let mut blocks = Blocks::default();
        while !opcode_to_solve.is_empty() {
            unresolved_opcodes.clear();
            let mut stalled = true;
            let mut opcode_not_solvable = None;
            for opcode in &opcode_to_solve {
                let resolution = match opcode {
                    Opcode::Arithmetic(expr) => ArithmeticSolver::solve(initial_witness, expr),
                    Opcode::BlackBoxFuncCall(bb_func) => {
                        Self::solve_black_box_function_call(initial_witness, bb_func)
                    }
                    Opcode::Directive(directive) => {
                        // Self::solve_directives(initial_witness, directive).map(|possible_log| {
                        //     if let Some(solved_log) = possible_log {
                        //         logs.push(solved_log)
                        //     }
                        // })
                        Self::solve_directives(initial_witness, directive)
                    }
                    Opcode::Block(block) | Opcode::ROM(block) | Opcode::RAM(block) => {
                        blocks.solve(block.id, &block.trace, initial_witness)
                    }
                };
                match resolution {
                    Ok(OpcodeResolution::Solved) => {
                        stalled = false;
                    }
                    Ok(OpcodeResolution::InProgress) => {
                        stalled = false;
                        unresolved_opcodes.push(opcode.clone());
                    }
                    Ok(OpcodeResolution::Stalled(not_solvable)) => {
                        if opcode_not_solvable.is_none() {
                            // we keep track of the first unsolvable opcode
                            opcode_not_solvable = Some(not_solvable);
                        }
                        // We push those opcodes not solvable to the back as
                        // it could be because the opcodes are out of order, i.e. this assignment
                        // relies on a later opcodes' results
                        unresolved_opcodes.push(opcode.clone());
                    }
                    Err(OpcodeResolutionError::OpcodeNotSolvable(_)) => {
                        unreachable!("ICE - Result should have been converted to GateResolution")
                    }
                    Err(err) => return Err(err),
                }
            }
            if stalled && !unresolved_opcodes.is_empty() {
                return Err(OpcodeResolutionError::OpcodeNotSolvable(
                    opcode_not_solvable
                        .expect("infallible: cannot be stalled and None at the same time"),
                ));
            }
            std::mem::swap(&mut opcode_to_solve, &mut unresolved_opcodes);
        }
        Ok(())
    }

    fn solve_black_box_function_call(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<OpcodeResolution, OpcodeResolutionError>;

    // Check if all of the inputs to the function have assignments
    // Returns true if all of the inputs have been assigned
    fn all_func_inputs_assigned(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> bool {
        // This call to .any returns true, if any of the witnesses do not have assignments
        // We then use `!`, so it returns false if any of the witnesses do not have assignments
        !func_call.inputs.iter().any(|input| !initial_witness.contains_key(&input.witness))
    }

    fn solve_directives(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        directive: &Directive,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        match pwg::directives::solve_directives(initial_witness, directive) {
            Ok(_) => Ok(OpcodeResolution::Solved),
            Err(OpcodeResolutionError::OpcodeNotSolvable(unsolved)) => {
                Ok(OpcodeResolution::Stalled(unsolved))
            }
            Err(err) => Err(err),
        }
    }
}

pub trait SmartContract {
    // TODO: Allow a backend to support multiple smart contract platforms

    /// Takes an ACIR circuit, the number of witnesses and the number of public inputs
    /// Then returns an Ethereum smart contract
    ///
    /// XXX: This will be deprecated in future releases for the above method.
    /// This deprecation may happen in two stages:
    /// The first stage will remove `num_witnesses` and `num_public_inputs` parameters.
    /// If we cannot avoid `num_witnesses`, it can be added into the Circuit struct.
    #[deprecated]
    fn eth_contract_from_cs(&self, circuit: Circuit) -> String;

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
