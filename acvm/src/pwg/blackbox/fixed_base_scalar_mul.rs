use acir::{
    circuit::{opcodes::FunctionInput, OpcodeLabel},
    native_types::{Witness, WitnessMap},
};

use crate::{
    pwg::{insert_value, witness_to_value, OpcodeResolution, OpcodeResolutionError},
    BlackBoxFunctionSolver,
};

pub(super) fn fixed_base_scalar_mul(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    input: FunctionInput,
    outputs: (Witness, Witness),
    opcode_idx: OpcodeLabel,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let scalar = witness_to_value(initial_witness, input.witness)?;

    let (pub_x, pub_y) = backend.fixed_base_scalar_mul(scalar)?;

    insert_value(&outputs.0, pub_x, initial_witness, opcode_idx)?;
    insert_value(&outputs.1, pub_y, initial_witness, opcode_idx)?;

    Ok(OpcodeResolution::Solved)
}
