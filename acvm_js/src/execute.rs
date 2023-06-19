use acvm::{
    acir::{
        circuit::{opcodes::FunctionInput, Circuit},
        native_types::{Witness, WitnessMap},
        BlackBoxFunc,
    },
    pwg::{
        insert_value, witness_to_value, Blocks, OpcodeResolution, OpcodeResolutionError,
        PartialWitnessGeneratorStatus,
    },
    FieldElement, PartialWitnessGenerator,
};

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{
    barretenberg::{pedersen::Pedersen, scalar_mul::ScalarMul, schnorr::SchnorrSig, Barretenberg},
    foreign_calls::ForeignCallHandler,
    JsWitnessMap,
};

struct SimulatedBackend {
    blackbox_vendor: Barretenberg,
}

impl SimulatedBackend {
    async fn initialize() -> SimulatedBackend {
        let blackbox_vendor = Barretenberg::new().await;
        SimulatedBackend { blackbox_vendor }
    }
}

impl PartialWitnessGenerator for SimulatedBackend {
    fn schnorr_verify(
        &self,
        initial_witness: &mut WitnessMap,
        public_key_x: &FunctionInput,
        public_key_y: &FunctionInput,
        signature: &[FunctionInput],
        message: &[FunctionInput],
        output: &Witness,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        // In barretenberg, if the signature fails, then the whole thing fails.

        let pub_key_x = witness_to_value(initial_witness, public_key_x.witness)?.to_be_bytes();
        let pub_key_y = witness_to_value(initial_witness, public_key_y.witness)?.to_be_bytes();

        let pub_key_bytes: Vec<u8> = pub_key_x.iter().copied().chain(pub_key_y.to_vec()).collect();
        let pub_key: [u8; 64] = pub_key_bytes.try_into().map_err(|v: Vec<u8>| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                BlackBoxFunc::SchnorrVerify,
                format!("expected pubkey size {} but received {}", 64, v.len()),
            )
        })?;

        let signature_bytes: Vec<u8> = signature
            .iter()
            .map(|sig_elem| {
                witness_to_value(initial_witness, sig_elem.witness).map(|witness_value| {
                    *witness_value.to_be_bytes().last().expect("byte array is never empty")
                })
            })
            .collect::<Result<_, _>>()?;

        let sig_s = signature_bytes[0..32].try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                BlackBoxFunc::SchnorrVerify,
                format!("signature should be 64 bytes long, found only {} bytes", signature.len()),
            )
        })?;
        let sig_e = signature_bytes[32..64].try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                BlackBoxFunc::SchnorrVerify,
                format!("signature should be 64 bytes long, found only {} bytes", signature.len()),
            )
        })?;

        let message_bytes: Vec<u8> = message
            .iter()
            .map(|message_elem| {
                witness_to_value(initial_witness, message_elem.witness).map(|witness_value| {
                    *witness_value.to_be_bytes().last().expect("byte array is never empty")
                })
            })
            .collect::<Result<_, _>>()?;

        let valid_signature = self
            .blackbox_vendor
            .verify_signature(pub_key, sig_s, sig_e, &message_bytes)
            .map_err(|err| {
                OpcodeResolutionError::BlackBoxFunctionFailed(
                    BlackBoxFunc::SchnorrVerify,
                    err.to_string(),
                )
            })?;
        if !valid_signature {
            dbg!("signature has failed to verify");
        }

        insert_value(output, FieldElement::from(valid_signature), initial_witness)?;
        Ok(OpcodeResolution::Solved)
    }

    fn pedersen(
        &self,
        initial_witness: &mut WitnessMap,
        inputs: &[FunctionInput],
        // Assumed to be `0`
        _domain_separator: u32,
        outputs: &[Witness],
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        let scalars: Result<Vec<_>, _> =
            inputs.iter().map(|input| witness_to_value(initial_witness, input.witness)).collect();
        let scalars: Vec<_> = scalars?.into_iter().cloned().collect();

        let (res_x, res_y) = self.blackbox_vendor.encrypt(scalars).map_err(|err| {
            OpcodeResolutionError::BlackBoxFunctionFailed(BlackBoxFunc::Pedersen, err.to_string())
        })?;
        insert_value(&outputs[0], res_x, initial_witness)?;
        insert_value(&outputs[1], res_y, initial_witness)?;
        Ok(OpcodeResolution::Solved)
    }

    fn fixed_base_scalar_mul(
        &self,
        initial_witness: &mut WitnessMap,
        input: &FunctionInput,
        outputs: &[Witness],
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        let scalar = witness_to_value(initial_witness, input.witness)?;

        let (pub_x, pub_y) = self.blackbox_vendor.fixed_base(scalar).map_err(|err| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                BlackBoxFunc::FixedBaseScalarMul,
                err.to_string(),
            )
        })?;

        insert_value(&outputs[0], pub_x, initial_witness)?;
        insert_value(&outputs[1], pub_y, initial_witness)?;
        Ok(OpcodeResolution::Solved)
    }
}

/// Executes an ACIR circuit to generate the solved witness from the initial witness.
///
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} initial_witness - The initial witness map defining all of the inputs to `circuit`..
/// @param {ForeignCallHandler} foreign_call_handler - A callback to process any foreign calls from the circuit.
/// @returns {WitnessMap} The solved witness calculated by executing the circuit on the provided inputs.
#[wasm_bindgen(js_name = executeCircuit, skip_jsdoc)]
pub async fn execute_circuit(
    circuit: Vec<u8>,
    initial_witness: JsWitnessMap,
    _foreign_call_handler: ForeignCallHandler,
) -> Result<JsWitnessMap, JsValue> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let mut witness_map = WitnessMap::from(initial_witness);

    let backend = SimulatedBackend::initialize().await;
    let mut blocks = Blocks::default();
    let mut opcodes = circuit.opcodes;

    loop {
        let solver_status = acvm::pwg::solve(&backend, &mut witness_map, &mut blocks, opcodes)
            .map_err(|err| err.to_string())?;

        match solver_status {
            PartialWitnessGeneratorStatus::Solved => break,
            PartialWitnessGeneratorStatus::RequiresOracleData {
                required_oracle_data: _,
                unsolved_opcodes,
                unresolved_brillig_calls: _,
            } => {
                // TODO: add handling for `Brillig` opcodes.

                // Use new opcodes as returned by ACVM.
                opcodes = unsolved_opcodes;
            }
        }
    }

    Ok(witness_map.into())
}
