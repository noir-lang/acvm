use crate::{pwg::witness_to_value, pwg::OpcodeResolution, OpcodeResolutionError};
use acir::{circuit::opcodes::FunctionInput, native_types::WitnessMap};

pub fn solve_range_opcode(
    initial_witness: &mut WitnessMap,
    input: &FunctionInput,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let w_value = witness_to_value(initial_witness, input.witness)?;
    if w_value.num_bits() > input.num_bits {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain);
    }
    Ok(OpcodeResolution::Solved)
}
