use acir::circuit::opcode::BlackBoxFuncCall;
use acir::native_types::Witness;
use acir_field::FieldElement;
use std::collections::BTreeMap;

pub struct LogicSolver;

impl LogicSolver {
    /// Derives the rest of the witness based on the initial low level variables
    fn solve_logic_gate(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        a: &Witness,
        b: &Witness,
        result: Witness,
        num_bits: u32,
        is_xor_gate: bool,
    ) -> bool {
        let w_l = initial_witness.get(a);
        let w_r = initial_witness.get(b);

        let (w_l_value, w_r_value) = match (w_l, w_r) {
            (Some(w_l_value), Some(w_r_value)) => (w_l_value, w_r_value),
            (_, _) => return false,
        };

        if is_xor_gate {
            let assignment = w_l_value.xor(w_r_value, num_bits);
            initial_witness.insert(result, assignment);
        } else {
            let assignment = w_l_value.and(w_r_value, num_bits);
            initial_witness.insert(result, assignment);
        }
        true
    }

    pub fn solve_and_gate(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gate: &BlackBoxFuncCall,
    ) -> bool {
        let (a, b, result, num_bits) = extract_input_output(gate);
        LogicSolver::solve_logic_gate(initial_witness, &a, &b, result, num_bits, false)
    }
    pub fn solve_xor_gate(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gate: &BlackBoxFuncCall,
    ) -> bool {
        let (a, b, result, num_bits) = extract_input_output(gate);
        LogicSolver::solve_logic_gate(initial_witness, &a, &b, result, num_bits, true)
    }
}
// TODO: Is there somewhere else that we can put this?
// TODO: extraction methods are needed for some opcodes like logic and range
pub(crate) fn extract_input_output(gc: &BlackBoxFuncCall) -> (Witness, Witness, Witness, u32) {
    let a = &gc.inputs[0];
    let b = &gc.inputs[1];
    let result = &gc.outputs[0];

    // The num_bits variable should be the same for all witnesses
    assert_eq!(
        a.num_bits, b.num_bits,
        "number of bits specified for each input must be the same"
    );

    let num_bits = a.num_bits;

    (a.witness, b.witness, *result, num_bits)
}
