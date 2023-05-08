use super::{insert_value, witness_to_value};
use crate::{pwg::OpcodeResolution, OpcodeResolutionError};
use acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
    native_types::Witness,
    FieldElement,
};
use std::collections::BTreeMap;

pub fn solve_logic_opcode(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    func_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    match func_call {
        BlackBoxFuncCall::AND { lhs, rhs, output } => {
            LogicSolver::solve_and_gate(initial_witness, lhs, rhs, output)
        }
        BlackBoxFuncCall::XOR { lhs, rhs, output } => {
            LogicSolver::solve_xor_gate(initial_witness, lhs, rhs, output)
        }
        _ => Err(OpcodeResolutionError::UnexpectedOpcode(
            "logic opcode",
            func_call.get_black_box_func(),
        )),
    }
}

pub struct LogicSolver;

impl LogicSolver {
    /// Derives the rest of the witness based on the initial low level variables
    fn solve_logic_gate(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        a: &Witness,
        b: &Witness,
        result: &Witness,
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
        insert_value(result, assignment, initial_witness)?;
        Ok(OpcodeResolution::Solved)
    }

    pub fn solve_and_gate(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        lhs: &FunctionInput,
        rhs: &FunctionInput,
        output: &Witness,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        assert_eq!(
            lhs.num_bits, rhs.num_bits,
            "number of bits specified for each input must be the same"
        );

        LogicSolver::solve_logic_gate(
            initial_witness,
            &lhs.witness,
            &rhs.witness,
            output,
            lhs.num_bits,
            false,
        )
    }
    pub fn solve_xor_gate(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        lhs: &FunctionInput,
        rhs: &FunctionInput,
        output: &Witness,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        assert_eq!(
            lhs.num_bits, rhs.num_bits,
            "number of bits specified for each input must be the same"
        );

        LogicSolver::solve_logic_gate(
            initial_witness,
            &lhs.witness,
            &rhs.witness,
            output,
            lhs.num_bits,
            true,
        )
    }
}
