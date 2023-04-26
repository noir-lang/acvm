use crate::{pwg::witness_to_value, OpcodeResolution, OpcodeResolutionError};
use acir::{circuit::opcodes::BlackBoxFuncCall, native_types::Witness, BlackBoxFunc, FieldElement};
use std::collections::BTreeMap;

pub fn solve_range_opcode(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    func_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    // TODO: this consistency check can be moved to a general function
    let defined_input_size = BlackBoxFunc::RANGE
        .definition()
        .input_size
        .fixed_size()
        .expect("infallible: input for range gate is fixed");

    let num_arguments = func_call.inputs.len();
    if num_arguments != defined_input_size as usize {
        return Err(OpcodeResolutionError::IncorrectNumFunctionArguments(
            defined_input_size as usize,
            BlackBoxFunc::RANGE,
            num_arguments,
        ));
    }

    // For the range constraint, we know that the input size should be one
    assert_eq!(defined_input_size, 1);

    let input = func_call.inputs.first().expect("infallible: checked that input size is 1");

    let w_value = witness_to_value(initial_witness, input.witness)?;
    if w_value.num_bits() > input.num_bits {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain);
    }
    Ok(OpcodeResolution::Solved)
}
