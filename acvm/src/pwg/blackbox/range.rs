use crate::{pwg::witness_to_value, OpcodeResolutionError};
use acir::{
    circuit::{opcodes::FunctionInput, OpcodeLabel},
    native_types::WitnessMap,
};

pub(super) fn solve_range_opcode(
    initial_witness: &mut WitnessMap,
    input: &FunctionInput,
) -> Result<(), OpcodeResolutionError> {
    let w_value = witness_to_value(initial_witness, input.witness)?;
    if w_value.num_bits() > input.num_bits {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_label: OpcodeLabel::Unresolved,
        });
    }
    Ok(())
}
