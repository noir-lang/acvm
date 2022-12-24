use crate::compiler::GeneralOptimiser;
use acir::circuit::{Circuit, Gate};

pub struct R1CSOptimiser {
    acir: Circuit,
}

impl R1CSOptimiser {
    pub fn new(acir: Circuit) -> Self {
        Self { acir }
    }
    // R1CS optimisations uses the general optimiser.
    // TODO: We could possibly make sure that all polynomials are at most degree-2
    pub fn optimise(self) -> Circuit {
        let optimised_arith_gates: Vec<_> = self
            .acir
            .gates
            .into_iter()
            .map(|gate| match gate {
                Gate::Arithmetic(arith) => Gate::Arithmetic(GeneralOptimiser::optimise(arith)),
                other_gates => other_gates,
            })
            .collect();

        Circuit {
            // The general optimiser may remove enough gates that a witness is no longer used
            // however, we cannot decrement the number of witnesses, as that
            // would require a linear scan over all gates in order to decrement all witness indices
            // above the witness which was removed
            current_witness_index: self.acir.current_witness_index,
            gates: optimised_arith_gates,
            public_inputs: self.acir.public_inputs,
        }
    }
}
