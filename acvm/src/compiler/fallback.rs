use crate::{CustomGate, Language};
use acir::{
    circuit::{opcodes::BlackBoxFuncCall, Circuit, Opcode},
    native_types::Expression,
    BlackBoxFunc,
};

//Acir pass which replace unsupported gates using arithmetic fallback
pub fn fallback(acir: &Circuit, np_language: &Language) -> Circuit {
    let mut fallback_gates = Vec::new();
    let mut witness_idx = acir.current_witness_index + 1;
    for g in &acir.opcodes {
        if !np_language.supports_opcode(g) {
            let gates = gate_fallback(g, &mut witness_idx);
            fallback_gates.extend(gates);
        } else {
            fallback_gates.push(g.clone());
        }
    }

    Circuit {
        current_witness_index: witness_idx,
        opcodes: fallback_gates,
        public_inputs: acir.public_inputs.clone(),
    }
}

fn gate_fallback(gate: &Opcode, witness_idx: &mut u32) -> Vec<Opcode> {
    let mut gadget_gates = Vec::new();
    match gate {
        Opcode::BlackBoxFuncCall(gc) if gc.name == BlackBoxFunc::RANGE => {
            // TODO: add consistency checks in one place
            // TODO: we aren't checking that range gate should have one input
            let input = &gc.inputs[0];
            // Note there are no outputs because range produces no outputs
            *witness_idx = stdlib::fallback::range(
                Expression::from(&input.witness),
                input.num_bits,
                *witness_idx,
                &mut gadget_gates,
            );
        }
        Opcode::BlackBoxFuncCall(gc) if gc.name == BlackBoxFunc::AND => {
            let (lhs, rhs, result, num_bits) = crate::pwg::logic::extract_input_output(&gc);
            *witness_idx = stdlib::fallback::and(
                Expression::from(&lhs),
                Expression::from(&rhs),
                result,
                num_bits,
                *witness_idx,
                &mut gadget_gates,
            );
        }
        Opcode::BlackBoxFuncCall(gc) if gc.name == BlackBoxFunc::XOR => {
            let (lhs, rhs, result, num_bits) = crate::pwg::logic::extract_input_output(&gc);
            *witness_idx = stdlib::fallback::xor(
                Expression::from(&lhs),
                Expression::from(&rhs),
                result,
                num_bits,
                *witness_idx,
                &mut gadget_gates,
            );
        }
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall { name, .. }) => {
            unreachable!("Missing alternative for opcode {}", name)
        }
        _ => todo!("no fallback for gate {:?}", gate),
    }

    gadget_gates
}
