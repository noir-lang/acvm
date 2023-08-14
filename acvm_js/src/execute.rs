use acvm::{
    acir::circuit::Circuit,
    pwg::{ACVMStatus, ACVM},
};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    foreign_call::{resolve_brillig, ForeignCallHandler},
    JsWitnessMap,
};

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
    foreign_call_handler: ForeignCallHandler,
) -> Result<JsWitnessMap, js_sys::JsString> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");

    #[allow(deprecated)]
    let backend = acvm::blackbox_solver::BarretenbergSolver::initialize().await;
    let mut acvm = ACVM::new(backend, circuit.opcodes, initial_witness.into());

    loop {
        let solver_status = acvm.solve();

        match solver_status {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => {
                unreachable!("Execution should not stop while in `InProgress` state.")
            }
            ACVMStatus::Failure(error) => return Err(error.to_string().into()),
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let result = resolve_brillig(&foreign_call_handler, &foreign_call).await?;

                acvm.resolve_pending_foreign_call(result);
            }
        }
    }

    let witness_map = acvm.finalize();
    Ok(witness_map.into())
}
