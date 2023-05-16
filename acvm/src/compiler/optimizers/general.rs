use acir::{
    native_types::{Expression, Witness},
    FieldElement,
};
use indexmap::IndexMap;

pub(crate) struct GeneralOpt;
impl GeneralOpt {
    pub(crate) fn optimize(gate: Expression) -> Expression {
        // XXX: Perhaps this optimization can be done on the fly
        let gate = remove_zero_coefficients(gate);
        simplify_mul_terms(gate)
    }
}

// Remove all terms with zero as a coefficient
fn remove_zero_coefficients(mut gate: Expression) -> Expression {
    // Check the mul terms
    gate.mul_terms.retain(|(scale, _, _)| !scale.is_zero());
    // Check the linear combination terms
    gate.linear_combinations.retain(|(scale, _)| !scale.is_zero());
    gate
}

// Simplifies all mul terms with the same bi-variate variables
fn simplify_mul_terms(mut gate: Expression) -> Expression {
    let mut hash_map: IndexMap<(Witness, Witness), FieldElement> = IndexMap::new();

    // Canonicalize the ordering of the multiplication, lets just order by variable name
    for (scale, w_l, w_r) in gate.mul_terms.clone().into_iter() {
        let mut pair = vec![w_l, w_r];
        // Sort using rust sort algorithm
        pair.sort();

        *hash_map.entry((pair[0], pair[1])).or_insert_with(FieldElement::zero) += scale;
    }

    gate.mul_terms = hash_map.into_iter().map(|((w_l, w_r), scale)| (scale, w_l, w_r)).collect();
    gate
}
