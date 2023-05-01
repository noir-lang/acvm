use super::{insert_value, witness_to_value};
use crate::{OpcodeResolution, OpcodeResolutionError};
use acir::{
    circuit::opcodes::BlackBoxFuncCall,
    native_types::{Witness, WitnessMap},
    BlackBoxFunc,
};

pub fn solve_logic_opcode(
    initial_witness: &mut WitnessMap,
    func_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    match func_call.name {
        BlackBoxFunc::AND => LogicSolver::solve_and_gate(initial_witness, func_call),
        BlackBoxFunc::XOR => LogicSolver::solve_xor_gate(initial_witness, func_call),
        _ => Err(OpcodeResolutionError::UnexpectedOpcode("logic opcode", func_call.name)),
    }
}

pub struct LogicSolver;

impl LogicSolver {
    /// Derives the rest of the witness based on the initial low level variables
    fn solve_logic_gate(
        initial_witness: &mut WitnessMap,
        a: &Witness,
        b: &Witness,
        result: Witness,
        num_bits: u32,
        is_xor_gate: bool,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        let w_l_value = witness_to_value(initial_witness, *a)?;
        let w_r_value = witness_to_value(initial_witness, *b)?;

        let assignment = if is_xor_gate {
            w_l_value.xor(w_r_value, num_bits)
        } else {
            w_l_value.and(w_r_value, num_bits)
        };
        insert_value(&result, assignment, initial_witness)?;
        Ok(OpcodeResolution::Solved)
    }

    pub fn solve_and_gate(
        initial_witness: &mut WitnessMap,
        gate: &BlackBoxFuncCall,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        let (a, b, result, num_bits) = extract_input_output(gate);
        LogicSolver::solve_logic_gate(initial_witness, &a, &b, result, num_bits, false)
    }
    pub fn solve_xor_gate(
        initial_witness: &mut WitnessMap,
        gate: &BlackBoxFuncCall,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        let (a, b, result, num_bits) = extract_input_output(gate);
        LogicSolver::solve_logic_gate(initial_witness, &a, &b, result, num_bits, true)
    }
}
// TODO: Is there somewhere else that we can put this?
// TODO: extraction methods are needed for some opcodes like logic and range
pub(crate) fn extract_input_output(
    bb_func_call: &BlackBoxFuncCall,
) -> (Witness, Witness, Witness, u32) {
    let a = &bb_func_call.inputs[0];
    let b = &bb_func_call.inputs[1];
    let result = &bb_func_call.outputs[0];

    // The num_bits variable should be the same for all witnesses
    assert_eq!(a.num_bits, b.num_bits, "number of bits specified for each input must be the same");

    let num_bits = a.num_bits;

    (a.witness, b.witness, *result, num_bits)
}
