use acir::{
    circuit::{
        brillig::BrilligOutputs, directives::Directive, opcodes::UnsupportedMemoryOpcode, Circuit,
        Opcode, OpcodeLabel,
    },
    native_types::{Expression, Witness},
    BlackBoxFunc, FieldElement,
};
use indexmap::IndexMap;
use thiserror::Error;

use crate::Language;

// The various passes that we can use over ACIR
mod optimizers;
mod transformers;

use optimizers::{GeneralOptimizer, RangeOptimizer};
use transformers::{CSatTransformer, FallbackTransformer, R1CSTransformer};

#[derive(PartialEq, Eq, Debug, Error)]
pub enum CompileError {
    #[error("The blackbox function {0} is not supported by the backend and acvm does not have a fallback implementation")]
    UnsupportedBlackBox(BlackBoxFunc),
    #[error("The opcode {0} is not supported by the backend and acvm does not have a fallback implementation")]
    UnsupportedMemoryOpcode(UnsupportedMemoryOpcode),
}

/// Applies [`ProofSystemCompiler`][crate::ProofSystemCompiler] specific optimizations to a [`Circuit`].
pub fn compile(
    acir: Circuit,
    np_language: Language,
    is_opcode_supported: impl Fn(&Opcode) -> bool,
) -> Result<(Circuit, Vec<OpcodeLabel>), CompileError> {
    // Instantiate the optimizer.
    // Currently the optimizer and reducer are one in the same
    // for CSAT

    // Track original opcode label throughout the transformation passes of the compilation
    // by applying the modifications done to the circuit opcodes and also to the opcode_label (delete and insert)
    let opcode_labels: Vec<OpcodeLabel> = acir.initial_opcode_labels();

    // Fallback transformer pass
    let (acir, opcode_label) =
        FallbackTransformer::transform(acir, is_opcode_supported, opcode_labels)?;

    // General optimizer pass
    let mut opcodes: Vec<Opcode> = Vec::new();
    for opcode in acir.opcodes {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                opcodes.push(Opcode::Arithmetic(GeneralOptimizer::optimize(arith_expr)))
            }
            other_gate => opcodes.push(other_gate),
        };
    }
    let acir = Circuit { opcodes, ..acir };

    // Range optimization pass
    let range_optimizer = RangeOptimizer::new(acir);
    let (acir, opcode_label) = range_optimizer.replace_redundant_ranges(opcode_label);

    let mut transformer = match &np_language {
        crate::Language::R1CS => {
            let transformer = R1CSTransformer::new(acir);
            return Ok((transformer.transform(), opcode_label));
        }
        crate::Language::PLONKCSat { width } => {
            let mut csat = CSatTransformer::new(*width);
            for value in acir.circuit_arguments() {
                csat.mark_solvable(value);
            }
            csat
        }
    };

    // TODO: the code below is only for CSAT transformer
    // TODO it may be possible to refactor it in a way that we do not need to return early from the r1cs
    // TODO or at the very least, we could put all of it inside of CSatOptimizer pass

    let mut new_opcode_labels = Vec::with_capacity(opcode_label.len());
    // Optimize the arithmetic gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut transformed_gates = Vec::new();

    let mut next_witness_index = acir.current_witness_index + 1;
    // maps a normalized expression to the intermediate variable which represents the expression, along with its 'norm'
    // the 'norm' is simply the value of the first non zero coefficient in the expression, taken from the linear terms, or quadratic terms if there is none.
    let mut intermediate_variables: IndexMap<Expression, (FieldElement, Witness)> = IndexMap::new();
    for (index, opcode) in acir.opcodes.iter().enumerate() {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                let len = intermediate_variables.len();

                let arith_expr = transformer.transform(
                    arith_expr.clone(),
                    &mut intermediate_variables,
                    &mut next_witness_index,
                );

                // Update next_witness counter
                next_witness_index += (intermediate_variables.len() - len) as u32;
                let mut new_gates = Vec::new();
                for (g, (norm, w)) in intermediate_variables.iter().skip(len) {
                    // de-normalize
                    let mut intermediate_gate = g * *norm;
                    // constrain the intermediate gate to the intermediate variable
                    intermediate_gate.linear_combinations.push((-FieldElement::one(), *w));
                    intermediate_gate.sort();
                    new_gates.push(intermediate_gate);
                }
                new_gates.push(arith_expr);
                for gate in new_gates {
                    new_opcode_labels.push(opcode_label[index]);
                    transformed_gates.push(Opcode::Arithmetic(gate));
                }
            }
            Opcode::BlackBoxFuncCall(func) => {
                match func {
                    acir::circuit::opcodes::BlackBoxFuncCall::AND { output, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::XOR { output, .. } => {
                        transformer.mark_solvable(*output)
                    }
                    acir::circuit::opcodes::BlackBoxFuncCall::RANGE { .. } => (),
                    acir::circuit::opcodes::BlackBoxFuncCall::SHA256 { outputs, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::Keccak256 { outputs, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::Keccak256VariableLength {
                        outputs,
                        ..
                    }
                    | acir::circuit::opcodes::BlackBoxFuncCall::RecursiveAggregation {
                        output_aggregation_object: outputs,
                        ..
                    }
                    | acir::circuit::opcodes::BlackBoxFuncCall::Blake2s { outputs, .. } => {
                        for witness in outputs {
                            transformer.mark_solvable(*witness);
                        }
                    }
                    acir::circuit::opcodes::BlackBoxFuncCall::FixedBaseScalarMul {
                        outputs,
                        ..
                    }
                    | acir::circuit::opcodes::BlackBoxFuncCall::Pedersen { outputs, .. } => {
                        transformer.mark_solvable(outputs.0);
                        transformer.mark_solvable(outputs.1)
                    }
                    acir::circuit::opcodes::BlackBoxFuncCall::HashToField128Security {
                        output,
                        ..
                    }
                    | acir::circuit::opcodes::BlackBoxFuncCall::EcdsaSecp256k1 { output, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::EcdsaSecp256r1 { output, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::SchnorrVerify { output, .. } => {
                        transformer.mark_solvable(*output)
                    }
                }

                new_opcode_labels.push(opcode_label[index]);
                transformed_gates.push(opcode.clone());
            }
            Opcode::Directive(directive) => {
                match directive {
                    Directive::Invert { result, .. } => {
                        transformer.mark_solvable(*result);
                    }
                    Directive::Quotient(quotient_directive) => {
                        transformer.mark_solvable(quotient_directive.q);
                        transformer.mark_solvable(quotient_directive.r);
                    }
                    Directive::ToLeRadix { b, .. } => {
                        for w in b {
                            transformer.mark_solvable(*w);
                        }
                    }
                    Directive::PermutationSort { bits, .. } => {
                        for w in bits {
                            transformer.mark_solvable(*w);
                        }
                    }
                    Directive::Log(_) => (),
                }
                new_opcode_labels.push(opcode_label[index]);
                transformed_gates.push(opcode.clone());
            }
            Opcode::MemoryInit { .. } => {
                // `MemoryInit` does not write values to the `WitnessMap`
            }
            Opcode::MemoryOp { op, .. } => {
                for (_, w1, w2) in &op.value.mul_terms {
                    transformer.mark_solvable(*w1);
                    transformer.mark_solvable(*w2);
                }
                for (_, w) in &op.value.linear_combinations {
                    transformer.mark_solvable(*w);
                }
            }

            Opcode::Block(_) | Opcode::ROM(_) | Opcode::RAM(_) => {
                unimplemented!("Stepwise execution is not compatible with {}", opcode.name())
            }
            Opcode::Brillig(brillig) => {
                for output in &brillig.outputs {
                    match output {
                        BrilligOutputs::Simple(w) => transformer.mark_solvable(*w),
                        BrilligOutputs::Array(v) => {
                            for w in v {
                                transformer.mark_solvable(*w);
                            }
                        }
                    }
                }
                new_opcode_labels.push(opcode_label[index]);
                transformed_gates.push(opcode.clone());
            }
        }
    }

    let current_witness_index = next_witness_index - 1;
    Ok((
        Circuit {
            current_witness_index,
            opcodes: transformed_gates,
            // The optimizer does not add new public inputs
            private_parameters: acir.private_parameters,
            public_parameters: acir.public_parameters,
            return_values: acir.return_values,
        },
        new_opcode_labels,
    ))
}
