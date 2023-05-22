#![warn(unused_crate_dependencies)]
#![warn(unreachable_pub)]

// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub mod compiler;
pub mod pwg;

use acir::{
    circuit::{opcodes::FunctionInput, Circuit, Opcode},
    native_types::{Witness, WitnessMap},
};
use core::fmt::Debug;
use pwg::{OpcodeResolution, OpcodeResolutionError};

// We re-export async-trait so consumers can attach it to their impl
pub use async_trait::async_trait;

// re-export acir
pub use acir;
pub use acir::FieldElement;

/// Supported NP complete languages
/// This might need to be in ACIR instead
#[derive(Debug, Clone)]
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
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
#[async_trait(?Send)]
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

/// This component will generate the backend specific output for each [`Opcode::BlackBoxFuncCall`].
///
/// Returns an [`OpcodeResolutionError`] if the backend does not support the given [`Opcode::BlackBoxFuncCall`].
pub trait PartialWitnessGenerator {
    fn aes128(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        outputs: &[Witness],
    ) -> Result<OpcodeResolution, OpcodeResolutionError>;
    fn schnorr_verify(
        &self,
        initial_witness: &mut WitnessMap,
        public_key_x: &FunctionInput,
        public_key_y: &FunctionInput,
        signature: &[FunctionInput],
        message: &[FunctionInput],
        output: &Witness,
    ) -> Result<OpcodeResolution, OpcodeResolutionError>;
    fn pedersen(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        outputs: &[Witness],
    ) -> Result<OpcodeResolution, OpcodeResolutionError>;
    fn fixed_base_scalar_mul(
        &self,
        initial_witness: &mut WitnessMap,
        input: &FunctionInput,
        outputs: &[Witness],
    ) -> Result<OpcodeResolution, OpcodeResolutionError>;
}

pub trait SmartContract {
    /// The Error type returned by failed function calls in the SmartContract trait.
    type Error: std::error::Error;

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
    type Error: std::error::Error;

    /// The NPC language that this proof system directly accepts.
    /// It is possible for ACVM to transpile to different languages, however it is advised to create a new backend
    /// as this in most cases will be inefficient. For this reason, we want to throw a hard error
    /// if the language and proof system does not line up.
    fn np_language(&self) -> Language;

    // Returns true if the backend supports the selected opcode
    fn supports_opcode(&self, opcode: &Opcode) -> bool;

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
