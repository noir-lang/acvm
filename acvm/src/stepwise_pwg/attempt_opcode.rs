use std::collections::BTreeMap;

use acir::{
    circuit::{opcodes::BlackBoxFuncCall, Opcode},
    native_types::Witness,
    FieldElement,
};

use crate::{
    pwg::{self, arithmetic::ArithmeticSolver},
    OpcodeResolutionError,
};

use super::attempt_blackbox::{attempt_black_box, AttemptBlackBoxOutcome};

pub enum AttemptOpcodeOutcome {
    Solved,
    Skipped(Opcode),
    Blocked(BlackBoxFuncCall),
    Err(OpcodeResolutionError),
}

pub fn attempt_opcode(
    witness_skeleton: &mut BTreeMap<Witness, FieldElement>,
    opcode: Opcode,
) -> AttemptOpcodeOutcome {
    if let Opcode::BlackBoxFuncCall(bb_func_call) = opcode {
        return match attempt_black_box(witness_skeleton, &bb_func_call) {
            Ok(AttemptBlackBoxOutcome::Solved) => AttemptOpcodeOutcome::Solved,
            Ok(AttemptBlackBoxOutcome::Skipped) => {
                AttemptOpcodeOutcome::Skipped(Opcode::BlackBoxFuncCall(bb_func_call))
            }
            Ok(AttemptBlackBoxOutcome::Blocked) => AttemptOpcodeOutcome::Blocked(bb_func_call),
            Err(err) => AttemptOpcodeOutcome::Err(err),
        };
    }
    let result = match &opcode {
        Opcode::Arithmetic(expr) => ArithmeticSolver::solve(witness_skeleton, &expr),
        Opcode::Directive(directive) => {
            pwg::directives::solve_directives(witness_skeleton, &directive)
        }
        Opcode::BlackBoxFuncCall(_) => panic!("Handled by above `if let`"),
    };
    match result {
        Ok(_) => AttemptOpcodeOutcome::Solved,
        Err(err) => AttemptOpcodeOutcome::Err(err),
    }
}
