#[allow(deprecated)]
use acvm::{
    acir::circuit::Circuit,
    blackbox_solver::BarretenbergSolver,
    pwg::{ACVMStatus, ErrorLocation, OpcodeResolutionError, ACVM},
};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    foreign_call::{resolve_brillig, ForeignCallHandler},
    JsWitnessMap,
};

#[wasm_bindgen]
#[allow(deprecated)]
pub struct WasmBlackBoxFunctionSolver(BarretenbergSolver);

impl WasmBlackBoxFunctionSolver {
    async fn initialize() -> WasmBlackBoxFunctionSolver {
        #[allow(deprecated)]
        WasmBlackBoxFunctionSolver(BarretenbergSolver::initialize().await)
    }
}

#[wasm_bindgen(js_name = "createBlackBoxSolver")]
pub async fn create_black_box_solver() -> WasmBlackBoxFunctionSolver {
    WasmBlackBoxFunctionSolver::initialize().await
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
    foreign_call_handler: ForeignCallHandler,
) -> Result<JsWitnessMap, js_sys::JsString> {
    console_error_panic_hook::set_once();

    let solver = WasmBlackBoxFunctionSolver::initialize().await;

    execute_circuit_with_black_box_solver(&solver, circuit, initial_witness, foreign_call_handler)
        .await
}

/// Executes an ACIR circuit to generate the solved witness from the initial witness.
///
/// @param {&WasmBlackBoxFunctionSolver} solver - A black box solver.
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} initial_witness - The initial witness map defining all of the inputs to `circuit`..
/// @param {ForeignCallHandler} foreign_call_handler - A callback to process any foreign calls from the circuit.
/// @returns {WitnessMap} The solved witness calculated by executing the circuit on the provided inputs.
#[wasm_bindgen(js_name = executeCircuitWithBlackBoxSolver, skip_jsdoc)]
pub async fn execute_circuit_with_black_box_solver(
    solver: &WasmBlackBoxFunctionSolver,
    circuit: Vec<u8>,
    initial_witness: JsWitnessMap,
    foreign_call_handler: ForeignCallHandler,
) -> Result<JsWitnessMap, js_sys::JsString> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");

    let mut acvm = ACVM::new(&solver.0, circuit.opcodes, initial_witness.into());

    loop {
        let solver_status = acvm.solve();

        match solver_status {
            ACVMStatus::Solved => break,
            ACVMStatus::InProgress => {
                unreachable!("Execution should not stop while in `InProgress` state.")
            }
            ACVMStatus::Failure(error) => {
                let assert_message = match &error {
                    OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: ErrorLocation::Resolved(opcode_location),
                    }
                    | OpcodeResolutionError::IndexOutOfBounds {
                        opcode_location: ErrorLocation::Resolved(opcode_location),
                        ..
                    } => circuit.assert_messages.get(opcode_location).cloned(),
                    _ => None,
                };

                let error_string = match assert_message {
                    Some(assert_message) => format!("{}: {}", error, assert_message),
                    None => error.to_string(),
                };

                return Err(error_string.into());
            }
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let result = resolve_brillig(&foreign_call_handler, &foreign_call).await?;

                acvm.resolve_pending_foreign_call(result);
            }
        }
    }

    let witness_map = acvm.finalize();
    Ok(witness_map.into())
}
