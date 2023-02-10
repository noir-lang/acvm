use crate::compiler::GeneralOptimizer;
use acir::circuit::{Circuit, Opcode};

pub struct R1CSOptimizer {
    acir: Circuit,
}

impl R1CSOptimizer {
    pub fn new(acir: Circuit) -> Self {
        Self { acir }
    }
    // R1CS optimizations uses the general optimizer.
    // TODO: We could possibly make sure that all polynomials are at most degree-2
    pub fn optimize(self) -> Circuit {
        let optimized_arith_gates: Vec<_> = self
            .acir
            .opcodes
            .into_iter()
            .map(|gate| match gate {
                Opcode::Arithmetic(arith) => Opcode::Arithmetic(GeneralOptimizer::optimize(arith)),
                other_gates => other_gates,
            })
            .collect();

        Circuit {
            // The general optimizer may remove enough gates that a witness is no longer used
            // however, we cannot decrement the number of witnesses, as that
            // would require a linear scan over all gates in order to decrement all witness indices
            // above the witness which was removed
            current_witness_index: self.acir.current_witness_index,
            opcodes: optimized_arith_gates,
            public_inputs: self.acir.public_inputs,
        }
    }
}
