#![warn(unused_crate_dependencies)]
#![warn(unreachable_pub)]

// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub mod compiler;
pub mod pwg;

use acir::{
    circuit::{
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Circuit, Opcode,
    },
    native_types::{Expression, Witness, WitnessMap},
    BlackBoxFunc,
};
use core::fmt::Debug;
use thiserror::Error;

// We re-export async-trait so consumers can attach it to their impl
pub use async_trait::async_trait;

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
    #[error("expected {0} inputs for function {1}, but got {2}")]
    IncorrectNumFunctionArguments(usize, BlackBoxFunc, usize),
    #[error("failed to solve blackbox function: {0}, reason: {1}")]
    BlackBoxFunctionFailed(BlackBoxFunc, String),
}

pub trait Backend:
    SmartContract
    + ProofSystemCompiler
    + PartialWitnessGenerator
    + CommonReferenceString
    + Default
    + Debug
{
}

// Unfortunately, Rust doesn't natively allow async functions in traits yet.
// So we need to annotate our trait with this macro and backends need to attach the macro to their `impl`.
//
// For more details, see https://docs.rs/async-trait/latest/async_trait/
// and https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/
#[async_trait]
pub trait CommonReferenceString {
    /// The Error type returned by failed function calls in the CommonReferenceString trait.
    type Error: std::error::Error; // fully-qualified named because thiserror is `use`d at the top of the crate

    /// Provides the common reference string that is needed by other traits
    async fn generate_common_reference_string(
        &self,
        circuit: &Circuit,
    ) -> Result<Vec<u8>, Self::Error>;

    /// Updates a cached common reference string within the context of a circuit
    ///
    /// This function will be called if the common reference string has been cached previously
    /// and the backend can update it if necessary. This may happen if the common reference string
    /// contains fewer than the number of points needed by the circuit, or fails any other checks the backend
    /// must perform.
    ///
    /// If the common reference string doesn't need any updates, implementors can return the value passed.
    async fn update_common_reference_string(
        &self,
        common_reference_string: Vec<u8>,
        circuit: &Circuit,
    ) -> Result<Vec<u8>, Self::Error>;
}

/// This component will generate the backend specific output for
/// each OPCODE.
/// Returns an Error if the backend does not support that OPCODE
pub trait PartialWitnessGenerator {
    fn aes(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        outputs: &[Witness],
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn and(
        &self,
        initial_witness: &mut WitnessMap,
        lhs: &FunctionInput,
        rhs: &FunctionInput,
        output: &Witness,
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn xor(
        &self,
        initial_witness: &mut WitnessMap,
        lhs: &FunctionInput,
        rhs: &FunctionInput,
        output: &Witness,
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn range(
        &self,
        initial_witness: &mut WitnessMap,
        input: &FunctionInput,
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn sha256(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        outputs: &[Witness],
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn blake2s(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        outputs: &[Witness],
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn compute_merkle_root(
        &self,
        initial_witness: &mut WitnessMap,
        leaf: &FunctionInput,
        index: &FunctionInput,
        hash_path: &[FunctionInput],
        output: &Witness,
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn schnorr_verify(
        &self,
        initial_witness: &mut WitnessMap,
        public_key_x: &FunctionInput,
        public_key_y: &FunctionInput,
        signature: &[FunctionInput],
        message: &[FunctionInput],
        output: &Witness,
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn pedersen(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        outputs: &[Witness],
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn hash_to_field_128_security(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        outputs: &Witness,
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn ecdsa_secp256k1(
        &self,
        initial_witness: &mut WitnessMap,
        public_key_x: &[FunctionInput],
        public_key_y: &[FunctionInput],
        signature: &[FunctionInput],
        message: &[FunctionInput],
        outputs: &Witness,
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn fixed_base_scalar_mul(
        &self,
        initial_witness: &mut WitnessMap,
        input: &FunctionInput,
        outputs: &[Witness],
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
    fn keccak256(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        outputs: &[Witness],
    ) -> Result<pwg::OpcodeResolution, OpcodeResolutionError>;
}

pub trait SmartContract {
    /// The Error type returned by failed function calls in the SmartContract trait.
    type Error: std::error::Error; // fully-qualified named because thiserror is `use`d at the top of the crate

    // TODO: Allow a backend to support multiple smart contract platforms

    /// Returns an Ethereum smart contract to verify proofs against a given common reference string and verification key.
    fn eth_contract_from_vk(
        &self,
        common_reference_string: &[u8],
        verification_key: &[u8],
    ) -> Result<String, Self::Error>;
}

pub trait ProofSystemCompiler {
    /// The Error type returned by failed function calls in the ProofSystemCompiler trait.
    type Error: std::error::Error; // fully-qualified named because thiserror is `use`d at the top of the crate

    /// The NPC language that this proof system directly accepts.
    /// It is possible for ACVM to transpile to different languages, however it is advised to create a new backend
    /// as this in most cases will be inefficient. For this reason, we want to throw a hard error
    /// if the language and proof system does not line up.
    fn np_language(&self) -> Language;

    // Returns true if the backend supports the selected black box function
    fn black_box_function_supported(&self, opcode: &BlackBoxFunc) -> bool;

    /// Returns the number of gates in a circuit
    fn get_exact_circuit_size(&self, circuit: &Circuit) -> Result<u32, Self::Error>;

    /// Generates a proving and verification key given the circuit description
    /// These keys can then be used to construct a proof and for its verification
    fn preprocess(
        &self,
        common_reference_string: &[u8],
        circuit: &Circuit,
    ) -> Result<(Vec<u8>, Vec<u8>), Self::Error>;

    /// Creates a Proof given the circuit description, the initial witness values, and the proving key
    /// It is important to note that the intermediate witnesses for black box functions will not generated
    /// This is the responsibility of the proof system.
    fn prove_with_pk(
        &self,
        common_reference_string: &[u8],
        circuit: &Circuit,
        witness_values: WitnessMap,
        proving_key: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;

    /// Verifies a Proof, given the circuit description, the circuit's public inputs, and the verification key
    fn verify_with_vk(
        &self,
        common_reference_string: &[u8],
        proof: &[u8],
        public_inputs: WitnessMap,
        circuit: &Circuit,
        verification_key: &[u8],
    ) -> Result<bool, Self::Error>;
}

/// Supported NP complete languages
/// This might need to be in ACIR instead
#[derive(Debug, Clone)]
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}

#[deprecated]
pub fn hash_constraint_system(cs: &Circuit) -> [u8; 32] {
    let mut bytes = Vec::new();
    cs.write(&mut bytes).expect("could not serialize circuit");

    use sha2::{digest::FixedOutput, Digest, Sha256};
    let mut hasher = Sha256::new();

    hasher.update(bytes);
    hasher.finalize_fixed().into()
}

#[deprecated]
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
        !matches!(opcode, Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AES { .. }) | Opcode::Block(_))
    }

    match language {
        Language::R1CS => r1cs_is_supported,
        Language::PLONKCSat { .. } => plonk_is_supported,
    }
}
