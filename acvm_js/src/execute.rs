use acvm::{
    acir::{
        circuit::{
            opcodes::{FunctionInput, OracleData},
            Circuit,
        },
        native_types::{Witness, WitnessMap},
        BlackBoxFunc,
    },
    pwg::{
        block::Blocks, witness_to_value, OpcodeResolution, OpcodeResolutionError,
        PartialWitnessGeneratorStatus,
    },
    FieldElement, PartialWitnessGenerator,
};

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{
    barretenberg::{pedersen::Pedersen, scalar_mul::ScalarMul, schnorr::SchnorrSig, Barretenberg},
    foreign_calls::{resolve_oracle, OracleCallback},
    JsWitnessMap,
};

#[derive(Default)]
struct SimulatedBackend {
    blackbox_vendor: Barretenberg,
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

        initial_witness.insert(*output, FieldElement::from(valid_signature));
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
        initial_witness.insert(outputs[0], res_x);
        initial_witness.insert(outputs[1], res_y);
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

        initial_witness.insert(outputs[0], pub_x);
        initial_witness.insert(outputs[1], pub_y);
        Ok(OpcodeResolution::Solved)
    }
}

/// Executes an ACIR circuit to generate the solved witness from the initial witness.
///
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} initial_witness - The initial witness map defining all of the inputs to `circuit`..
/// @param {OracleCallback} oracle_callback - A callback to process oracle calls from the circuit.
/// @returns {WitnessMap} The solved witness calculated by executing the circuit on the provided inputs.
#[wasm_bindgen(js_name = executeCircuit, skip_jsdoc)]
pub async fn execute_circuit(
    circuit: Vec<u8>,
    initial_witness: JsWitnessMap,
    oracle_callback: OracleCallback,
) -> Result<JsWitnessMap, JsValue> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let mut witness_map = WitnessMap::from(initial_witness);

    let backend = SimulatedBackend::default();
    let mut blocks = Blocks::default();
    let mut opcodes = circuit.opcodes;

    loop {
        let solver_status = acvm::pwg::solve(&backend, &mut witness_map, &mut blocks, opcodes)
            .map_err(|err| err.to_string())?;

        match solver_status {
            PartialWitnessGeneratorStatus::Solved => break,
            PartialWitnessGeneratorStatus::RequiresOracleData {
                required_oracle_data,
                unsolved_opcodes,
                unresolved_brillig_calls: _,
            } => {
                process_oracle_calls(&mut witness_map, &oracle_callback, required_oracle_data)
                    .await?;

                // TODO: add handling for `Brillig` opcodes.

                // Use new opcodes as returned by ACVM.
                opcodes = unsolved_opcodes;
            }
        }
    }

    Ok(witness_map.into())
}

/// Performs the foreign calls associated with [`unresolved_oracle_calls`][OracleData] and writes the results to [`witness_map`][WitnessMap].
async fn process_oracle_calls(
    witness_map: &mut WitnessMap,
    oracle_callback: &OracleCallback,
    unresolved_oracle_calls: Vec<OracleData>,
) -> Result<(), String> {
    // Perform all oracle queries
    let oracle_call_futures: Vec<_> = unresolved_oracle_calls
        .into_iter()
        .map(|oracle_call| resolve_oracle(oracle_callback, oracle_call))
        .collect();

    // Insert results into the witness map
    for oracle_call_future in oracle_call_futures {
        let resolved_oracle_call: OracleData = oracle_call_future.await.unwrap();
        for (i, witness_index) in resolved_oracle_call.outputs.iter().enumerate() {
            insert_value(witness_index, resolved_oracle_call.output_values[i], witness_map)
                .map_err(|err| err.to_string())?;
        }
    }

    Ok(())
}

fn insert_value(
    witness: &Witness,
    value_to_insert: FieldElement,
    initial_witness: &mut WitnessMap,
) -> Result<(), OpcodeResolutionError> {
    let optional_old_value = initial_witness.insert(*witness, value_to_insert);

    let old_value = match optional_old_value {
        Some(old_value) => old_value,
        None => return Ok(()),
    };

    if old_value != value_to_insert {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain);
    }

    Ok(())
}
