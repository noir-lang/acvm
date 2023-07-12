use std::{cmp::Ordering, collections::HashSet};

use acir::{
    native_types::{Expression, Witness},
    FieldElement,
};
use indexmap::IndexMap;

/// A transformer which processes any [`Expression`]s to break them up such that they
/// fit within the [`ProofSystemCompiler`][crate::ProofSystemCompiler]'s width.
///
/// This transformer is only used when targetting the [`PLONKCSat`][crate::Language::PLONKCSat] language.
///
/// This is done by creating intermediate variables to hold partial calculations and then combining them
/// to calculate the original expression.
// Should we give it all of the gates?
// Have a single transformer that you instantiate with a width, then pass many gates through
pub(crate) struct CSatTransformer {
    width: usize,
    /// Track the witness that can be solved
    solvable_witness: HashSet<Witness>,
}

impl CSatTransformer {
    // Configure the width for the optimizer
    pub(crate) fn new(width: usize) -> CSatTransformer {
        assert!(width > 2);

        CSatTransformer { width, solvable_witness: HashSet::new() }
    }

    /// Returns true if the equation 'expression=0' can be solved, and add the solved witness to set of solvable witness
    fn solvable_expression(&mut self, gate: &Expression) -> bool {
        let mut unresolved = Vec::new();
        for (_, w1, w2) in &gate.mul_terms {
            if !self.solvable_witness.contains(w1) {
                unresolved.push(w1);
                if !self.solvable_witness.contains(w2) {
                    return false;
                }
            }
            if !self.solvable_witness.contains(w2) {
                unresolved.push(w2);
                if !self.solvable_witness.contains(w1) {
                    return false;
                }
            }
        }
        for (_, w) in &gate.linear_combinations {
            if !self.solvable_witness.contains(w) {
                unresolved.push(w);
            }
        }
        if unresolved.len() == 1 {
            self.solvable(*unresolved[0]);
        }
        unresolved.len() <= 1
    }

    /// Adds the witness to set of solvable witness
    pub(crate) fn solvable(&mut self, witness: Witness) {
        self.solvable_witness.insert(witness);
    }

    // Still missing dead witness optimization.
    // To do this, we will need the whole set of arithmetic gates
    // I think it can also be done before the local optimization seen here, as dead variables will come from the user
    pub(crate) fn transform(
        &mut self,
        gate: Expression,
        intermediate_variables: &mut IndexMap<Expression, (FieldElement, Witness)>,
        num_witness: &mut u32,
    ) -> Expression {
        // Here we create intermediate variables and constrain them to be equal to any subset of the polynomial that can be represented as a full gate
        let gate = self.full_gate_scan_optimization(gate, intermediate_variables, num_witness);
        // The last optimization to do is to create intermediate variables in order to flatten the fan-in and the amount of mul terms
        // If a gate has more than one mul term. We may need an intermediate variable for each one. Since not every variable will need to link to
        // the mul term, we could possibly do it that way.
        // We wil call this a partial gate scan optimization which will result in the gates being able to fit into the correct width
        let mut gate =
            self.partial_gate_scan_optimization(gate, intermediate_variables, num_witness);
        gate.sort();
        self.solvable_expression(&gate);
        gate
    }

    // This optimization will search for combinations of terms which can be represented in a single arithmetic gate
    // Case 1 : qM * wL * wR + qL * wL + qR * wR + qO * wO + qC
    // This polynomial does not require any further optimizations, it can be safely represented in one gate
    // ie a polynomial with 1 mul(bi-variate) term and 3 (univariate) terms where 2 of those terms match the bivariate term
    // wL and wR, we can represent it in one gate
    // GENERALIZED for WIDTH: instead of the number 3, we use `WIDTH`
    //
    //
    // Case 2: qM * wL * wR + qL * wL + qR * wR + qO * wO + qC + qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    // This polynomial cannot be represented using one arithmetic gate.
    //
    // This algorithm will first extract the first full gate(if possible):
    // t = qM * wL * wR + qL * wL + qR * wR + qO * wO + qC
    //
    // The polynomial now looks like so t + qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    // This polynomial cannot be represented using one arithmetic gate.
    //
    // This algorithm will then extract the second full gate(if possible):
    // t2 = qM2 * wL2 * wR2 + qL * wL2 + qR * wR2 + qO * wO2 + qC2
    //
    // The polynomial now looks like so t + t2
    // We can no longer extract another full gate, hence the algorithm terminates. Creating two intermediate variables t and t2.
    // This stage of preprocessing does not guarantee that all polynomials can fit into a gate. It only guarantees that all full gates have been extracted from each polynomial
    fn full_gate_scan_optimization(
        &mut self,
        mut gate: Expression,
        intermediate_variables: &mut IndexMap<Expression, (FieldElement, Witness)>,
        num_witness: &mut u32,
    ) -> Expression {
        // We pass around this intermediate variable IndexMap, so that we do not create intermediate variables that we have created before
        // One instance where this might happen is t1 = wL * wR and t2 = wR * wL

        // First check that this is not a simple gate which does not need optimization
        //
        // If the gate only has one mul term, then this algorithm cannot optimize it any further
        // Either it can be represented in a single arithmetic equation or it's fan-in is too large and we need intermediate variables for those
        // large-fan-in optimization is not this algorithms purpose.
        // If the gate has 0 mul terms, then it is an add gate and similarly it can either fit into a single arithmetic gate or it has a large fan-in
        if gate.mul_terms.len() <= 1 {
            return gate;
        }

        // We now know that this gate has multiple mul terms and can possibly be simplified into multiple full gates
        // We need to create a (wl, wr) IndexMap and then check the simplified fan-in to verify if we have terms both with wl and wr
        // In general, we can then push more terms into the gate until we are at width-1 then the last variable will be the intermediate variable
        //

        // This will be our new gate which will be equal to `self` except we will have intermediate variables that will be constrained to any
        // subset of the terms that can be represented as full gates
        let mut new_gate = Expression::default();
        let mut mul_term_remains = Vec::new();
        for pair in gate.mul_terms {
            // We want to layout solvable intermediate variable, if we cannot solve one of the witness
            // that means the intermediate gate will not be immediatly solvable
            if !self.solvable_witness.contains(&pair.1) || !self.solvable_witness.contains(&pair.2)
            {
                mul_term_remains.push(pair);
                continue;
            }

            // Check if this pair is present in the simplified fan-in
            // We are assuming that the fan-in/fan-out has been simplified.
            // Note this function is not public, and can only be called within the optimize method, so this guarantee will always hold
            let index_wl =
                gate.linear_combinations.iter().position(|(_scale, witness)| *witness == pair.1);
            let index_wr =
                gate.linear_combinations.iter().position(|(_scale, witness)| *witness == pair.2);

            match (index_wl, index_wr) {
                (None, _) => {
                    // This means that the polynomial does not contain both terms
                    // Just push the Qm term as it cannot form a full gate
                    new_gate.mul_terms.push(pair);
                }
                (_, None) => {
                    // This means that the polynomial does not contain both terms
                    // Just push the Qm term as it cannot form a full gate
                    new_gate.mul_terms.push(pair);
                }
                (Some(x), Some(y)) => {
                    // This means that we can form a full gate with this Qm term

                    // First fetch the left and right wires which match the mul term
                    let left_wire_term = gate.linear_combinations[x];
                    let right_wire_term = gate.linear_combinations[y];

                    // Lets create an intermediate gate to store this full gate
                    //
                    let mut intermediate_gate = Expression::default();
                    intermediate_gate.mul_terms.push(pair);

                    // Add the left and right wires
                    intermediate_gate.linear_combinations.push(left_wire_term);
                    intermediate_gate.linear_combinations.push(right_wire_term);
                    // Remove the left and right wires so we do not re-add them
                    match x.cmp(&y) {
                        Ordering::Greater => {
                            gate.linear_combinations.remove(x);
                            gate.linear_combinations.remove(y);
                        }
                        Ordering::Less => {
                            gate.linear_combinations.remove(y);
                            gate.linear_combinations.remove(x);
                        }
                        Ordering::Equal => {
                            gate.linear_combinations.remove(x);
                            intermediate_gate.linear_combinations.pop();
                        }
                    }

                    // Now we have used up 2 spaces in our arithmetic gate. The width now dictates, how many more we can add
                    let mut remaining_space = self.width - 2 - 1; // We minus 1 because we need an extra space to contain the intermediate variable
                                                                  // Keep adding terms until we have no more left, or we reach the width
                    let mut remaining_linear_terms = Vec::new();
                    while remaining_space > 0 {
                        if let Some(wire_term) = gate.linear_combinations.pop() {
                            // Add this element into the new gate
                            if self.solvable_witness.contains(&wire_term.1) {
                                intermediate_gate.linear_combinations.push(wire_term);
                                remaining_space -= 1;
                            } else {
                                remaining_linear_terms.push(wire_term);
                            }
                        } else {
                            // No more usable elements left in the old gate
                            gate.linear_combinations = remaining_linear_terms;
                            break;
                        }
                    }
                    // Constraint this intermediate_gate to be equal to the temp variable by adding it into the IndexMap
                    // We need a unique name for our intermediate variable
                    // XXX: Another optimization, which could be applied in another algorithm
                    // If two gates have a large fan-in/out and they share a few common terms, then we should create intermediate variables for them
                    // Do some sort of subset matching algorithm for this on the terms of the polynomial

                    let inter_var = Self::get_or_create_intermediate_vars(
                        intermediate_variables,
                        intermediate_gate,
                        num_witness,
                    );

                    // Add intermediate variable to the new gate instead of the full gate
                    self.solvable_witness.insert(inter_var.1);
                    new_gate.linear_combinations.push(inter_var);
                }
            };
        }
        gate.mul_terms = mul_term_remains;

        // Add the rest of the elements back into the new_gate
        new_gate.mul_terms.extend(gate.mul_terms.clone());
        new_gate.linear_combinations.extend(gate.linear_combinations.clone());
        new_gate.q_c = gate.q_c;
        new_gate.sort();
        new_gate
    }

    /// Normalize an expression by dividing it by its first coefficient
    /// The first coefficient here means coefficient of the first linear term, or of the first quadratic term if no linear terms exist.
    /// The function panic if the input expression is constant
    fn normalize(mut expr: Expression) -> (FieldElement, Expression) {
        expr.sort();
        let a = if !expr.linear_combinations.is_empty() {
            expr.linear_combinations[0].0
        } else {
            expr.mul_terms[0].0
        };
        (a, &expr * a.inverse())
    }

    /// Get or generate a scaled intermediate witness which is equal to the provided expression
    /// The sets of previously generated witness and their (normalized) expression is cached in the intermediate_variables map
    /// If there is no cache hit, we generate a new witness (and add the expression to the cache)
    /// else, we return the cached witness along with the scaling factor so it is equal to the provided expression
    fn get_or_create_intermediate_vars(
        intermediate_variables: &mut IndexMap<Expression, (FieldElement, Witness)>,
        expr: Expression,
        num_witness: &mut u32,
    ) -> (FieldElement, Witness) {
        let (k, normalized_expr) = Self::normalize(expr);

        if intermediate_variables.contains_key(&normalized_expr) {
            let (l, iv) = intermediate_variables[&normalized_expr];
            (k / l, iv)
        } else {
            let inter_var = Witness(*num_witness);
            *num_witness += 1;
            // Add intermediate gate and variable to map
            intermediate_variables.insert(normalized_expr, (k, inter_var));
            (FieldElement::one(), inter_var)
        }
    }

    // A partial gate scan optimization aim to create intermediate variables in order to compress the polynomial
    // So that it fits within the given width
    // Note that this gate follows the full gate scan optimization.
    // We define the partial width as equal to the full width - 2.
    // This is because two of our variables cannot be used as they are linked to the multiplication terms
    // Example: qM1 * wL1 * wR2 + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // One thing to note is that the multiplication wires do not match any of the fan-in/out wires. This is guaranteed as we have
    // just completed the full gate optimization algorithm.
    //
    //Actually we can optimize in two ways here: We can create an intermediate variable which is equal to the fan-in terms
    // t = qL1 * wL3 + qR1 * wR4 -> width = 3
    // This `t` value can only use width - 1 terms
    // The gate now looks like: qM1 * wL1 * wR2 + t + qR2 * wR5+ qO1 * wO5 + qC
    // But this is still not acceptable since wR5 is not wR2, so we need another intermediate variable
    // t2 = t + qR2 * wR5
    //
    // The gate now looks like: qM1 * wL1 * wR2 + t2 + qO1 * wO5 + qC
    // This is still not good, so we do it one more time:
    // t3 = t2 + qO1 * wO5
    // The gate now looks like: qM1 * wL1 * wR2 + t3 + qC
    //
    // Another strategy is to create a temporary variable for the multiplier term and then we can see it as a term in the fan-in
    //
    // Same Example: qM1 * wL1 * wR2 + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // t = qM1 * wL1 * wR2
    // The gate now looks like: t + qL1 * wL3 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // Still assuming width3, we still need to use width-1 terms for the intermediate variables, however we can stop at an earlier stage because
    // the gate does not need the multiplier term to match with any of the fan-in terms
    // t2 = t + qL1 * wL3
    // The gate now looks like: t2 + qR1 * wR4+ qR2 * wR5 + qO1 * wO5 + qC
    // t3 = t2 + qR1 * wR4
    // The gate now looks like: t3 + qR2 * wR5 + qO1 * wO5 + qC
    // This took the same amount of gates, but which one is better when the width increases? Compute this and maybe do both optimizations
    // naming : partial_gate_mul_first_opt and partial_gate_fan_first_opt
    // Also remember that since we did full gate scan, there is no way we can have a non-zero mul term along with the wL and wR terms being non-zero
    //
    // Cases, a lot of mul terms, a lot of fan-in terms, 50/50
    fn partial_gate_scan_optimization(
        &self,
        mut gate: Expression,
        intermediate_variables: &mut IndexMap<Expression, (FieldElement, Witness)>,
        num_witness: &mut u32,
    ) -> Expression {
        // We will go for the easiest route, which is to convert all multiplications into additions using intermediate variables
        // Then use intermediate variables again to squash the fan-in, so that it can fit into the appropriate width

        // First check if this polynomial actually needs a partial gate optimization
        // There is the chance that it fits perfectly within the arithmetic gate
        if gate.fits_in_one_identity(self.width) {
            return gate;
        }

        // 2. Create Intermediate variables for the multiplication gates
        let mut mult_terms_remains = Vec::new();
        for mul_term in gate.mul_terms.clone().into_iter() {
            if self.solvable_witness.contains(&mul_term.1)
                && self.solvable_witness.contains(&mul_term.2)
            {
                let mut intermediate_gate = Expression::default();

                // Push mul term into the gate
                intermediate_gate.mul_terms.push(mul_term);
                // Get an intermediate variable which squashes the multiplication term
                let inter_var = Self::get_or_create_intermediate_vars(
                    intermediate_variables,
                    intermediate_gate,
                    num_witness,
                );

                // Add intermediate variable as a part of the fan-in for the original gate
                gate.linear_combinations.push(inter_var);
            } else {
                mult_terms_remains.push(mul_term);
            }
        }

        // Remove all of the mul terms as we have intermediate variables to represent them now
        gate.mul_terms = mult_terms_remains;

        // We now only have a polynomial with only fan-in/fan-out terms i.e. terms of the form Ax + By + Cd + ...
        // Lets create intermediate variables if all of them cannot fit into the width
        //
        // If the polynomial fits perfectly within the given width, we are finished
        if gate.linear_combinations.len() <= self.width {
            return gate;
        }

        // Stores the intermediate variables that are used to
        // reduce the fan in.
        let mut added = Vec::new();

        while gate.linear_combinations.len() > self.width {
            // Collect as many terms up to the given width-1 and constrain them to an intermediate variable
            let mut intermediate_gate = Expression::default();

            let mut linear_term_remains = Vec::new();

            for term in gate.linear_combinations {
                if self.solvable_witness.contains(&term.1)
                    && intermediate_gate.linear_combinations.len() < self.width - 1
                {
                    intermediate_gate.linear_combinations.push(term);
                } else {
                    linear_term_remains.push(term);
                }
            }
            gate.linear_combinations = linear_term_remains;
            let not_full = intermediate_gate.linear_combinations.len() < self.width - 1;
            if intermediate_gate.linear_combinations.len() > 1 {
                let inter_var = Self::get_or_create_intermediate_vars(
                    intermediate_variables,
                    intermediate_gate,
                    num_witness,
                );
                added.push(inter_var);
            }
            //intermediate gate is not full, but the gate still has too many terms
            if not_full && gate.linear_combinations.len() > self.width {
                dbg!(&gate.linear_combinations);
                unreachable!("ICE - could not reduce the expresion");
            }
        }

        // Add back the intermediate variables to
        // keep consistency with the original equation.
        gate.linear_combinations.extend(added);
        dbg!("should stpr");
        self.partial_gate_scan_optimization(gate, intermediate_variables, num_witness)
    }
}

#[test]
fn simple_reduction_smoke_test() {
    let a = Witness(0);
    let b = Witness(1);
    let c = Witness(2);
    let d = Witness(3);

    // a = b + c + d;
    let gate_a = Expression {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), a),
            (-FieldElement::one(), b),
            (-FieldElement::one(), c),
            (-FieldElement::one(), d),
        ],
        q_c: FieldElement::zero(),
    };

    let mut intermediate_variables: IndexMap<Expression, (FieldElement, Witness)> = IndexMap::new();

    let mut num_witness = 4;

    let mut optimizer = CSatTransformer::new(3);
    let got_optimized_gate_a =
        optimizer.transform(gate_a, &mut intermediate_variables, &mut num_witness);

    // a = b + c + d => a - b - c - d = 0
    // For width3, the result becomes:
    // a - b + e = 0
    // - c - d  - e = 0
    //
    // a - b + e = 0
    let e = Witness(4);
    let expected_optimized_gate_a = Expression {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), a),
            (-FieldElement::one(), b),
            (FieldElement::one(), e),
        ],
        q_c: FieldElement::zero(),
    };
    assert_eq!(expected_optimized_gate_a, got_optimized_gate_a);

    assert_eq!(intermediate_variables.len(), 1);

    // e = - c - d
    let expected_intermediate_gate = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(-FieldElement::one(), d), (-FieldElement::one(), c)],
        q_c: FieldElement::zero(),
    };
    let (_, normalized_gate) = CSatTransformer::normalize(expected_intermediate_gate);
    assert!(intermediate_variables.contains_key(&normalized_gate));
    assert_eq!(intermediate_variables[&normalized_gate].1, e);
}
