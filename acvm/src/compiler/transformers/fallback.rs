use crate::compiler::optimizers::CircuitSimplifier;

use super::super::CompileError;
use acir::{
    circuit::{opcodes::BlackBoxFuncCall, Circuit, Opcode, OpcodeLabel},
    native_types::Expression,
};

/// The initial transformer to act on a [`Circuit`]. This replaces any unsupported opcodes with
/// fallback implementations consisting of well supported opcodes.
pub(crate) struct FallbackTransformer;

impl FallbackTransformer {
    //ACIR pass which replace unsupported opcodes using arithmetic fallback
    pub(crate) fn transform(
        acir: Circuit,
        is_supported: impl Fn(&Opcode) -> bool,
        simplifier: &CircuitSimplifier,
        opcode_labels: Vec<OpcodeLabel>,
    ) -> Result<(Circuit, Vec<OpcodeLabel>), CompileError> {
        let mut acir_supported_opcodes = Vec::with_capacity(acir.opcodes.len());
        let mut new_opcode_labels = Vec::with_capacity(opcode_labels.len());
        let mut witness_idx = acir.current_witness_index + 1;
        // add opcodes for defining the witness that will be solved through simplification but must be kept
        for w in &simplifier.defined {
            acir_supported_opcodes.push(simplifier.define(w));
        }
        for (idx, opcode) in acir.opcodes.into_iter().enumerate() {
            if !simplifier.solved_gates.contains(&idx) {
                match &opcode {
                    Opcode::Arithmetic(_)
                    | Opcode::Directive(_)
                    | Opcode::Brillig(_)
                    | Opcode::Block(_)
                    | Opcode::ROM(_)
                    | Opcode::RAM(_) => {
                        // directive, arithmetic expression or blocks are handled by acvm
                        new_opcode_labels.push(opcode_labels[idx]);
                        acir_supported_opcodes.push(opcode);
                        continue;
                    }
                    Opcode::BlackBoxFuncCall(bb_func_call) => {
                        // We know it is an black box function. Now check if it is
                        // supported by the backend. If it is supported, then we can simply
                        // collect the opcode
                        if is_supported(&opcode) {
                            new_opcode_labels.push(opcode_labels[idx]);
                            acir_supported_opcodes.push(opcode);
                            continue;
                        } else {
                            // If we get here then we know that this black box function is not supported
                            // so we need to replace it with a version of the opcode which only uses arithmetic
                            // expressions
                            let (updated_witness_index, opcodes_fallback) =
                                Self::opcode_fallback(bb_func_call, witness_idx)?;
                            witness_idx = updated_witness_index;
                            new_opcode_labels
                                .extend(vec![opcode_labels[idx]; opcodes_fallback.len()]);
                            acir_supported_opcodes.extend(opcodes_fallback);
                        }
                    }
                }
            }
        }

        Ok((
            Circuit {
                current_witness_index: witness_idx,
                opcodes: acir_supported_opcodes,
                public_parameters: acir.public_parameters,
                return_values: acir.return_values,
                inputs: acir.inputs,
            },
            new_opcode_labels,
        ))
    }

    fn opcode_fallback(
        gc: &BlackBoxFuncCall,
        current_witness_idx: u32,
    ) -> Result<(u32, Vec<Opcode>), CompileError> {
        let (updated_witness_index, opcodes_fallback) = match gc {
            BlackBoxFuncCall::AND { lhs, rhs, output } => {
                assert_eq!(
                    lhs.num_bits, rhs.num_bits,
                    "number of bits specified for each input must be the same"
                );
                stdlib::fallback::and(
                    Expression::from(lhs.witness),
                    Expression::from(rhs.witness),
                    *output,
                    lhs.num_bits,
                    current_witness_idx,
                )
            }
            BlackBoxFuncCall::XOR { lhs, rhs, output } => {
                assert_eq!(
                    lhs.num_bits, rhs.num_bits,
                    "number of bits specified for each input must be the same"
                );
                stdlib::fallback::xor(
                    Expression::from(lhs.witness),
                    Expression::from(rhs.witness),
                    *output,
                    lhs.num_bits,
                    current_witness_idx,
                )
            }
            BlackBoxFuncCall::RANGE { input } => {
                // Note there are no outputs because range produces no outputs
                stdlib::fallback::range(
                    Expression::from(input.witness),
                    input.num_bits,
                    current_witness_idx,
                )
            }
            _ => {
                return Err(CompileError::UnsupportedBlackBox(gc.get_black_box_func()));
            }
        };

        Ok((updated_witness_index, opcodes_fallback))
    }
}
