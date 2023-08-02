#![warn(unreachable_pub)]

pub mod compiler;
pub mod pwg;

use acir::{
    circuit::{Circuit, Opcode},
    native_types::WitnessMap,
};
pub use blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};
use core::fmt::Debug;
use pwg::OpcodeResolutionError;

// We re-export async-trait so consumers can attach it to their impl
pub use async_trait::async_trait;

// re-export acir
pub use acir;
pub use acir::FieldElement;
// re-export brillig vm
pub use brillig_vm;
// re-export blackbox solver
pub use blackbox_solver;

/// Supported NP complete languages
/// This might need to be in ACIR instead
#[derive(Debug, Clone, Copy)]
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}

pub trait Backend:
    SmartContract
    + ProofSystemCompiler
    + BlackBoxFunctionSolver
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

pub trait SmartContract {
    /// The Error type returned by failed function calls in the SmartContract trait.
    type Error: std::error::Error;

    // TODO: Allow a backend to support multiple smart contract platforms

    /// Returns an Ethereum smart contract to verify proofs against a given common reference string and verification key.
    fn eth_contract_from_vk(
        &self,
        common_reference_string: &[u8],
        circuit: &Circuit,
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
    ///
    /// The `is_recursive` flag represents whether one wants to create proofs that are to be natively verified.
    /// A proof system may use a certain hash type for the Fiat-Shamir normally that is not hash friendly (such as keccak to enable Solidity verification),
    /// but may want to use a snark-friendly hash function when performing native verification.
    fn prove_with_pk(
        &self,
        common_reference_string: &[u8],
        circuit: &Circuit,
        witness_values: WitnessMap,
        proving_key: &[u8],
        is_recursive: bool,
    ) -> Result<Vec<u8>, Self::Error>;

    /// Verifies a Proof, given the circuit description, the circuit's public inputs, and the verification key
    ///
    /// The `is_recursive` flag represents whether one wants to verify proofs that are to be natively verified.
    /// The flag must match the `is_recursive` flag used to generate the proof passed into this method, otherwise verification will return false.
    fn verify_with_vk(
        &self,
        common_reference_string: &[u8],
        proof: &[u8],
        public_inputs: WitnessMap,
        circuit: &Circuit,
        verification_key: &[u8],
        is_recursive: bool,
    ) -> Result<bool, Self::Error>;

    /// When performing recursive aggregation in a circuit it is most efficient to use a proof formatted using a backend's native field.
    /// This method is exposed to enable backends to integrate a native recursion format and optimize their recursive circuits.
    fn proof_as_fields(
        &self,
        proof: &[u8],
        public_inputs: WitnessMap,
    ) -> Result<Vec<FieldElement>, Self::Error>;

    /// When performing recursive aggregation in a circuit it is most efficient to use a verification key formatted using a backend's native field.
    /// This method is exposed to enable backends to integrate a native recursion format and optimize their recursive circuits.
    fn vk_as_fields(
        &self,
        common_reference_string: &[u8],
        verification_key: &[u8],
    ) -> Result<(Vec<FieldElement>, FieldElement), Self::Error>;
}
