// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub mod compiler;
pub mod pwg;

use crate::pwg::arithmetic::ArithmeticSolver;
use acir::{
    circuit::{directives::Directive, opcodes::BlackBoxFuncCall, Circuit, Opcode},
    native_types::{Expression, Witness},
    BlackBoxFunc,
};
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
    #[error("compiler error: unreachable code")]
    UnreachableCode,
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

pub trait Backend: SmartContract + ProofSystemCompiler + PartialWitnessGenerator {}

/// This component will generate the backend specific output for
/// each OPCODE.
/// Returns an Error if the backend does not support that OPCODE
pub trait PartialWitnessGenerator {
    fn solve(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        mut opcodes: Vec<Opcode>,
    ) -> Result<(), OpcodeResolutionError> {
        let mut unresolved_opcodes: Vec<Opcode> = Vec::new();
        while !opcodes.is_empty() {
            unresolved_opcodes.clear();

            for opcode in &opcodes {
                let resolution = match opcode {
                    Opcode::Arithmetic(expr) => ArithmeticSolver::solve(initial_witness, expr),
                    Opcode::BlackBoxFuncCall(bb_func) => {
                        Self::solve_black_box_function_call(initial_witness, bb_func)
                    }
                    Opcode::Directive(directive) => {
                        Self::solve_directives(initial_witness, directive)
                    }
                };

                match resolution {
                    Ok(_) => {
                        // We do nothing in the happy case
                    }
                    Err(OpcodeResolutionError::OpcodeNotSolvable(_)) => {
                        // For opcode not solvable errors, we push those opcodes to the back as
                        // it could be because the opcodes are out of order, i.e. this assignment
                        // relies on a later opcodes' results
                        unresolved_opcodes.push(opcode.clone());
                    }
                    Err(err) => return Err(err),
                }
            }
            std::mem::swap(&mut opcodes, &mut unresolved_opcodes);
        }
        Ok(())
    }

    fn solve_black_box_function_call(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<(), OpcodeResolutionError>;

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
    ) -> Result<(), OpcodeResolutionError> {
        pwg::directives::solve_directives(initial_witness, directive)
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

    #[deprecated]
    fn prove_with_meta(
        &self,
        circuit: Circuit,
        witness_values: BTreeMap<Witness, FieldElement>,
    ) -> Vec<u8>;

    #[deprecated]
    fn verify_from_cs(
        &self,
        proof: &[u8],
        public_inputs: Vec<FieldElement>,
        circuit: Circuit,
    ) -> bool;

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

#[deprecated(
    note = "For backwards compatibility, this method allows you to derive _sensible_ defaults for black box function support based on the np language. \n Backends should simply specify what they support."
)]
// This is set to match the previous functionality that we had
// Where we could deduce what black box functions were supported
// by knowing the np complete language
pub fn default_is_black_box_supported(
    language: Language,
) -> compiler::transformers::IsBlackBoxSupported {
    // R1CS does not support any of the black box functions by default.
    // The compiler will replace those that it can -- ie range, xor, and
    fn r1cs_is_supported(_opcode: &BlackBoxFunc) -> bool {
        false
    }

    // PLONK supports most of the black box functions by default
    // The ones which are not supported, the acvm compiler will
    // attempt to transform into supported gates. If these are also not available
    // then a compiler error will be emitted.
    fn plonk_is_supported(opcode: &BlackBoxFunc) -> bool {
        !matches!(opcode, BlackBoxFunc::AES)
    }

    match language {
        Language::R1CS => r1cs_is_supported,
        Language::PLONKCSat { .. } => plonk_is_supported,
    }
}
