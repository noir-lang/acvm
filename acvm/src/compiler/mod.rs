use acir::{
    circuit::{Circuit, Opcode},
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

pub use optimizers::{CircuitSimplifier, SimplifyResult};

#[derive(PartialEq, Eq, Debug, Error)]
pub enum CompileError {
    #[error("The blackbox function {0} is not supported by the backend and acvm does not have a fallback implementation")]
    UnsupportedBlackBox(BlackBoxFunc),
}

/// Applies [`ProofSystemCompiler`][crate::ProofSystemCompiler] specific optimizations to a [`Circuit`].
pub fn compile(
    acir: Circuit,
    np_language: Language,
    is_opcode_supported: impl Fn(&Opcode) -> bool,
    simplifier: &CircuitSimplifier,
) -> Result<(Circuit, Vec<usize>), CompileError> {
    // Instantiate the optimizer.
    // Currently the optimizer and reducer are one in the same
    // for CSAT

    // Track original opcode index throughout the transformation passes of the compilation
    // by applying the modifications done to the circuit opcodes also to the opcode_idx (delete and insert)
    let opcode_idx: Vec<usize> = acir.default_opcode_idx();

    // Fallback transformer pass
    let (acir, opcode_idx) =
        FallbackTransformer::transform(acir, is_opcode_supported, simplifier, opcode_idx)?;

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
    let (acir, opcode_idx) = range_optimizer.replace_redundant_ranges(opcode_idx);

    let transformer = match &np_language {
        crate::Language::R1CS => {
            let transformer = R1CSTransformer::new(acir);
            return Ok((transformer.transform(), opcode_idx));
        }
        crate::Language::PLONKCSat { width } => CSatTransformer::new(*width),
    };

    // TODO: the code below is only for CSAT transformer
    // TODO it may be possible to refactor it in a way that we do not need to return early from the r1cs
    // TODO or at the very least, we could put all of it inside of CSatOptimizer pass

    let mut new_opcode_idx = Vec::with_capacity(opcode_idx.len());
    // Optimize the arithmetic gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut transformed_gates = Vec::new();

    let mut next_witness_index = acir.current_witness_index + 1;
    // maps a normalized expression to the intermediate variable which represents the expression, along with its 'norm'
    // the 'norm' is simply the value of the first non zero coefficient in the expression, taken from the linear terms, or quadratic terms if there is none.
    let mut intermediate_variables: IndexMap<Expression, (FieldElement, Witness)> = IndexMap::new();
    for (idx, opcode) in acir.opcodes.iter().enumerate() {
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
                new_gates.sort();
                for gate in new_gates {
                    new_opcode_idx.push(opcode_idx[idx]);
                    transformed_gates.push(Opcode::Arithmetic(gate));
                }
            }
            other_gate => {
                new_opcode_idx.push(opcode_idx[idx]);
                transformed_gates.push(other_gate.clone())
            }
        }
    }

    let current_witness_index = next_witness_index - 1;

    Ok((
        Circuit {
            current_witness_index,
            opcodes: transformed_gates,
            // The optimizer does not add new public inputs
            public_parameters: acir.public_parameters,
            return_values: acir.return_values,
        },
        new_opcode_idx,
    ))
}
