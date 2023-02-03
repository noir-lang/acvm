// The various passes that we can use over ACIR
pub mod fallback;
pub mod optimiser;

use crate::Language;
use acir::{
    circuit::{Circuit, Opcode},
    native_types::{Expression, Witness},
    BlackBoxFunc,
};
use indexmap::IndexMap;
use optimiser::{CSatOptimiser, GeneralOptimiser};
use thiserror::Error;

use self::{fallback::IsBlackBoxSupported, optimiser::R1CSOptimiser};

#[derive(PartialEq, Eq, Debug, Error)]
pub enum CompileError {
    #[error("The blackbox function {0} is not supported by the backend and acvm does not have a fallback implementation")]
    UnsupportedBlackBox(BlackBoxFunc),
}

pub fn compile(
    acir: Circuit,
    np_language: Language,
    is_black_box_supported: IsBlackBoxSupported,
) -> Result<Circuit, CompileError> {
    // Instantiate the optimiser.
    // Currently the optimiser and reducer are one in the same
    // for CSAT

    // Fallback pass
    let fallback = fallback::fallback(acir, is_black_box_supported)?;

    let optimiser = match &np_language {
        crate::Language::R1CS => {
            let optimiser = R1CSOptimiser::new(fallback);
            return Ok(optimiser.optimise());
        }
        crate::Language::PLONKCSat { width } => CSatOptimiser::new(*width),
    };

    // TODO: the code below is only for CSAT optimiser
    // TODO it may be possible to refactor it in a way that we do not need to return early from the r1cs
    // TODO or at the very least, we could put all of it inside of CSATOptimiser pass

    // Optimise the arithmetic gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut optimised_gates = Vec::new();

    let mut next_witness_index = fallback.current_witness_index + 1;
    for opcode in fallback.opcodes {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                let mut intermediate_variables: IndexMap<Witness, Expression> = IndexMap::new();

                let arith_expr =
                    optimiser.optimise(arith_expr, &mut intermediate_variables, next_witness_index);

                // Update next_witness counter
                next_witness_index += intermediate_variables.len() as u32;
                let mut new_gates = Vec::new();
                for (_, mut g) in intermediate_variables {
                    g.sort();
                    new_gates.push(g);
                }
                new_gates.push(arith_expr);
                new_gates.sort();
                for gate in new_gates {
                    optimised_gates.push(Opcode::Arithmetic(gate));
                }
            }
            other_gate => optimised_gates.push(other_gate),
        }
    }

    let current_witness_index = next_witness_index - 1;

    Ok(Circuit {
        current_witness_index,
        opcodes: optimised_gates,
        public_inputs: fallback.public_inputs, // The optimiser does not add public inputs
    })
}
