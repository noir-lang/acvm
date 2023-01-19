use acir::{
    circuit::{Circuit, Opcode, PublicInputs},
    native_types::{Expression, Witness},
    FieldElement,
};
use indexmap::IndexMap;

// Identifies order-1 univariate expressions and their solutions.
// aka expressions that constrain a witness to a constant.
// This check assume that general optimiser pass has been run such terms are normalised.
fn attempt_solve_for_witness(arith_expr: &Expression) -> Option<(Witness, FieldElement)> {
    if !arith_expr.mul_terms.is_empty() {
        return None;
    }
    if arith_expr.linear_combinations.len() != 1 {
        return None;
    }
    let linear_term = arith_expr.linear_combinations.first()?;
    let solution = -arith_expr.q_c / linear_term.0;
    Some((linear_term.1, solution))
}

// Substitutes in the solution for a witness and returns the simplified expression
fn simplify_with_solved_witness(
    arith_expr: &Expression,
    solved_witness: Witness,
    witness_value: FieldElement,
) -> Expression {
    let mut updated_arith_expr = Expression::default();
    for mul_term in &arith_expr.mul_terms {
        let (q, w_l, w_r) = *mul_term;
        if w_l == solved_witness && w_r == solved_witness {
            // Case 1: Witness squared
            // Term collapses into q_c
            updated_arith_expr.q_c += witness_value * witness_value * q
        } else if w_l == solved_witness || w_r == solved_witness {
            // Case 2: Bivariate
            // Term collapses into linear term
            let other_witness = if w_l == solved_witness { w_r } else { w_l };
            add_or_append_linear_term(
                &mut updated_arith_expr.linear_combinations,
                other_witness,
                q * witness_value,
            );
        } else {
            // Case 3: No match
            // Preserve term as is
            updated_arith_expr.mul_terms.push(*mul_term)
        }
    }
    for linear_term in &arith_expr.linear_combinations {
        let (q, w) = *linear_term;
        if w == solved_witness {
            // Case 1: Witness matches
            // Collapse into q_c
            updated_arith_expr.q_c += q * witness_value;
        } else {
            // Case 2: Doesn't match
            // Recombine into linear terms
            add_or_append_linear_term(&mut updated_arith_expr.linear_combinations, w, q);
        }
    }
    updated_arith_expr.q_c += arith_expr.q_c;
    updated_arith_expr
}

// Upsert addition of a linear term
fn add_or_append_linear_term(
    linear_combinations: &mut Vec<(FieldElement, Witness)>,
    witness: Witness,
    q: FieldElement,
) {
    let existing_linear_term = linear_combinations
        .iter_mut()
        .find(|linear_term| linear_term.1 == witness);
    match existing_linear_term {
        Some(linear_term) => {
            linear_term.0 += q;
        }
        None => {
            linear_combinations.push((q, witness));
        }
    }
}

// Builds a map of witnesses to the expression that use them
fn index_opcodes_by_witness(acir: &Circuit) -> IndexMap<Witness, Vec<usize>> {
    let mut opcode_idx_by_witness: IndexMap<Witness, Vec<usize>> = IndexMap::new();
    for (opcode_idx, opcode) in acir.opcodes.iter().enumerate() {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                for witness in arith_expr.get_witnesses() {
                    let existing_expression_idxs = opcode_idx_by_witness.get_mut(&witness);
                    match existing_expression_idxs {
                        Some(existing_expression_idxs) => {
                            existing_expression_idxs.push(opcode_idx);
                        }
                        None => {
                            opcode_idx_by_witness.insert(witness, vec![opcode_idx]);
                        }
                    }
                }
            }
            _ => (),
        }
    }
    opcode_idx_by_witness
}

// Applies a solved witness, and checks if more solutions become known as a result.
// If so, any subsequent solutions are applied.
fn apply_simplification_recursive(
    acir: &mut Circuit,
    opcode_idx_by_witness: &IndexMap<Witness, Vec<usize>>,
    solved_witness: Witness,
    witness_value: FieldElement,
    idx_of_solved_opcode: usize,
) {
    if let Some(opcode_idxs) = opcode_idx_by_witness.get(&solved_witness) {
        for idx in opcode_idxs {
            if *idx == idx_of_solved_opcode {
                continue;
            }
            let opcode = &mut acir.opcodes[*idx];
            if let Opcode::Arithmetic(arith_expr) = opcode {
                let updated_arith_expr =
                    simplify_with_solved_witness(&arith_expr, solved_witness, witness_value);
                arith_expr.mul_terms = updated_arith_expr.mul_terms;
                arith_expr.linear_combinations = updated_arith_expr.linear_combinations;
                arith_expr.q_c = updated_arith_expr.q_c;

                if let Some((next_solved_witness, next_witness_value)) =
                    attempt_solve_for_witness(&arith_expr)
                {
                    apply_simplification_recursive(
                        acir,
                        opcode_idx_by_witness,
                        next_solved_witness,
                        next_witness_value,
                        *idx,
                    )
                }
            }
        }
    }
}

// The BackPropagateSolutionsOptimiser scans the circuit for expressions that are solvable for a witness
// contained within. Once a solution is found the, it is substituted into any other expressions that contain it.
// Those simplified expressions are then re-checked to see if they are now solvable, and thus the process
// repeats until no solutions are found.
pub struct BackPropagateSolutionsOptimiser;

impl BackPropagateSolutionsOptimiser {
    pub fn optimise(acir: &Circuit) -> Circuit {
        let mut result = acir.clone();
        let opcode_idxs_by_witness = index_opcodes_by_witness(&result);
        for (opcode_idx, opcode) in acir.opcodes.iter().enumerate() {
            if let Opcode::Arithmetic(arith_expr) = opcode {
                if let Some((witness, solve_value)) = attempt_solve_for_witness(arith_expr) {
                    apply_simplification_recursive(
                        &mut result,
                        &opcode_idxs_by_witness,
                        witness,
                        solve_value,
                        opcode_idx,
                    );
                }
            }
        }
        result
    }
}

#[test]
fn test_attempt_solve_for_witness() {
    let arith_expr = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(FieldElement::one(), Witness(0))],
        q_c: FieldElement::one(),
    };
    let result = attempt_solve_for_witness(&arith_expr);
    assert_eq!(result, Some((Witness(0), -FieldElement::one())));

    let arith_expr = Expression {
        mul_terms: vec![(FieldElement::one(), Witness(0), Witness(0))],
        linear_combinations: vec![(FieldElement::one(), Witness(1))],
        q_c: FieldElement::zero(),
    };
    let result = attempt_solve_for_witness(&arith_expr);
    assert_eq!(result, None);
}

#[test]
fn test_index_mut_expressions_by_witness() {
    let circuit = Circuit {
        current_witness_index: 4,
        opcodes: vec![
            Opcode::Arithmetic(Expression {
                mul_terms: vec![(FieldElement::one(), Witness(0), Witness(0))],
                linear_combinations: vec![(FieldElement::one(), Witness(1))],
                q_c: FieldElement::zero(),
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![(FieldElement::one(), Witness(1), Witness(2))],
                linear_combinations: vec![(FieldElement::one(), Witness(3))],
                q_c: FieldElement::zero(),
            }),
        ],
        public_inputs: PublicInputs(vec![Witness(0)]),
    };
    let result = index_opcodes_by_witness(&circuit);
    let expected_result = IndexMap::from([
        (Witness(0), vec![0]),
        (Witness(1), vec![0, 1]),
        (Witness(2), vec![1]),
        (Witness(3), vec![1]),
    ]);
    assert_eq!(result, expected_result);
}

#[test]
fn simplify() {
    let arith_expr = Expression {
        mul_terms: vec![(FieldElement::one(), Witness(0), Witness(0))],
        linear_combinations: vec![(FieldElement::one(), Witness(0))],
        q_c: FieldElement::zero(),
    };
    let expected_result = Expression {
        mul_terms: vec![],
        linear_combinations: vec![],
        q_c: FieldElement::one() + FieldElement::one(),
    };
    let result = simplify_with_solved_witness(&arith_expr, Witness(0), FieldElement::one());
    assert_eq!(result, expected_result);

    let arith_expr = Expression {
        mul_terms: vec![(FieldElement::one(), Witness(0), Witness(1))],
        linear_combinations: vec![(FieldElement::one(), Witness(0))],
        q_c: FieldElement::zero(),
    };
    let expected_result = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(FieldElement::one(), Witness(1))],
        q_c: FieldElement::one(),
    };
    let result = simplify_with_solved_witness(&arith_expr, Witness(0), FieldElement::one());
    assert_eq!(result, expected_result);
}

#[test]
fn test_simplify_recursive() {
    let circuit = Circuit {
        current_witness_index: 2,
        opcodes: vec![
            Opcode::Arithmetic(Expression {
                mul_terms: vec![(FieldElement::one(), Witness(0), Witness(0))],
                linear_combinations: vec![(FieldElement::one(), Witness(1))],
                q_c: FieldElement::zero(),
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), Witness(0))],
                q_c: -FieldElement::one(),
            }),
        ],
        public_inputs: PublicInputs(vec![Witness(0)]),
    };
    let expected_result = Circuit {
        current_witness_index: 2,
        opcodes: vec![
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), Witness(1))],
                q_c: FieldElement::one(),
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), Witness(0))],
                q_c: -FieldElement::one(),
            }),
        ],
        public_inputs: PublicInputs(vec![Witness(0)]),
    };
    assert_eq!(
        BackPropagateSolutionsOptimiser::optimise(&circuit),
        expected_result
    );
}
