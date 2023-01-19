use super::super::CompileError;
use acir::{
    circuit::{opcodes::BlackBoxFuncCall, Circuit, Opcode},
    native_types::Expression,
    BlackBoxFunc,
};

// A predicate that returns true if the black box function is supported
pub type IsBlackBoxSupported = fn(&BlackBoxFunc) -> bool;

pub struct FallbackTransformer;

impl FallbackTransformer {
    //ACIR pass which replace unsupported opcodes using arithmetic fallback
    pub fn transform(
        acir: Circuit,
        is_supported: IsBlackBoxSupported,
    ) -> Result<Circuit, CompileError> {
        let mut acir_supported_opcodes = Vec::with_capacity(acir.opcodes.len());

        let mut witness_idx = acir.current_witness_index + 1;

        for opcode in acir.opcodes {
            let bb_func_call = match &opcode {
                Opcode::Arithmetic(_) | Opcode::Directive(_) => {
                    // If it is not a black box function, then it is a directive or
                    // an arithmetic expression which are always supported
                    acir_supported_opcodes.push(opcode);
                    continue;
                }
                Opcode::BlackBoxFuncCall(bb_func_call) => {
                    // We know it is an black box function. Now check if it is
                    // supported by the backend. If it is supported, then we can simply
                    // collect the opcode
                    if is_supported(&bb_func_call.name) {
                        acir_supported_opcodes.push(opcode);
                        continue;
                    }
                    bb_func_call
                }
            };

            // If we get here then we know that this black box function is not supported
            // so we need to replace it with a version of the opcode which only uses arithmetic
            // expressions
            let (updated_witness_index, opcodes_fallback) =
                Self::opcode_fallback(bb_func_call, witness_idx)?;
            witness_idx = updated_witness_index;

            acir_supported_opcodes.extend(opcodes_fallback);
        }

        Ok(Circuit {
            current_witness_index: witness_idx,
            opcodes: acir_supported_opcodes,
            public_inputs: acir.public_inputs,
        })
    }

    fn opcode_fallback(
        gc: &BlackBoxFuncCall,
        current_witness_idx: u32,
    ) -> Result<(u32, Vec<Opcode>), CompileError> {
        let (updated_witness_index, opcodes_fallback) = match gc.name {
            BlackBoxFunc::AND => {
                let (lhs, rhs, result, num_bits) = crate::pwg::logic::extract_input_output(gc);
                stdlib::fallback::and(
                    Expression::from(&lhs),
                    Expression::from(&rhs),
                    result,
                    num_bits,
                    current_witness_idx,
                )
            }
            BlackBoxFunc::XOR => {
                let (lhs, rhs, result, num_bits) = crate::pwg::logic::extract_input_output(gc);
                stdlib::fallback::xor(
                    Expression::from(&lhs),
                    Expression::from(&rhs),
                    result,
                    num_bits,
                    current_witness_idx,
                )
            }
            BlackBoxFunc::RANGE => {
                // TODO: add consistency checks in one place
                // TODO: we aren't checking that range gate should have one input
                let input = &gc.inputs[0];
                // Note there are no outputs because range produces no outputs
                stdlib::fallback::range(
                    Expression::from(&input.witness),
                    input.num_bits,
                    current_witness_idx,
                )
            }
            _ => {
                return Err(CompileError::UnsupportedBlackBox(gc.name));
            }
        };

        Ok((updated_witness_index, opcodes_fallback))
    }
}
