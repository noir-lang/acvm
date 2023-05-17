use std::collections::{BTreeMap, BTreeSet, HashSet};

use acir::{
    circuit::{
        directives::{Directive, QuotientDirective},
        opcodes::BlackBoxFuncCall,
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

use num_bigint::BigUint;
use num_traits::{FromPrimitive, Zero};

use crate::pwg::arithmetic::MulTerm;

#[derive(PartialEq, Eq)]
pub enum SimplifyResult {
    /// Opcode cannot be simplified
    Unresolved,
    /// Opcode is simplified into the boxed opcode
    Replace(Box<Opcode>),
    /// Opcode is redundant and can be removed
    Solved,
    /// Opcode is redundant, and solves a witness
    SolvedWitness(Witness),
    /// Opcode is not satisfied
    UnsatisfiedConstrain(usize),
}

pub struct CircuitSimplifier {
    /// Number of witness in the ABI
    abi_len: u32,
    solved: BTreeMap<Witness, FieldElement>,
    /// List of solved witness that should be defined with an Arithmetic gate
    pub defined: HashSet<Witness>,
    /// Index of the Arithmetic gate that defines a witness
    def_info: BTreeMap<Witness, usize>,
    /// Min of the solved witness definition
    min_use: usize,
    // Indexes of the solved opcodes
    pub solved_gates: BTreeSet<usize>,
    /// Indexes of input gates
    def_gates: BTreeSet<usize>,
}

impl CircuitSimplifier {
    pub fn new(abi_len: u32) -> CircuitSimplifier {
        CircuitSimplifier {
            abi_len,
            solved: BTreeMap::new(),
            defined: HashSet::new(),
            def_info: BTreeMap::new(),
            min_use: usize::MAX,
            solved_gates: BTreeSet::new(),
            def_gates: BTreeSet::new(),
        }
    }

    pub fn use_witness(&mut self, w: Witness, gate_idx: usize, first: bool) {
        if first && !self.def_info.contains_key(&w) {
            self.def_info.insert(w, gate_idx);
        }
    }

    pub fn is_abi(&self, w: Witness) -> bool {
        w.0 < self.abi_len
    }

    pub fn is_solved(&self, w: &Witness) -> bool {
        self.solved.contains_key(w)
    }

    pub fn insert(&mut self, w: Witness, f: FieldElement, gate_idx: usize) -> SimplifyResult {
        if !self.def_info.contains_key(&w) {
            if self.is_abi(w) && w.as_usize() < self.min_use {
                self.min_use = w.as_usize();
            }
        } else {
            let def = self.def_info[&w];
            if def < self.min_use {
                self.min_use = def;
            }
        }

        if self.contains(w) {
            if self.solved[&w] != f {
                return SimplifyResult::UnsatisfiedConstrain(gate_idx);
            }
            SimplifyResult::Solved
        } else {
            self.solved.insert(w, f);
            SimplifyResult::SolvedWitness(w)
        }
    }

    pub fn contains(&self, w: Witness) -> bool {
        self.solved.contains_key(&w)
    }

    // Generate an Arithmetic gate which set witness to its value
    pub fn define(&self, w: &Witness) -> Opcode {
        let mut a = Expression::from(*w);
        a.q_c = -self.solved[w];
        Opcode::Arithmetic(a)
    }

    // Simplify a gate and propagate the solved witness onto the previous gates, as long as it can solve some witness
    pub fn simplify(&mut self, gates: &mut Vec<Opcode>) -> SimplifyResult {
        let mut first = true;
        let mut solved = true;
        self.min_use = gates.len() - 1;
        while solved {
            solved = false;
            let mut i = gates.len() - 1;
            while i >= self.min_use {
                let gate = &gates[i];
                match self.simplify_opcode(gate, i, first) {
                    SimplifyResult::Unresolved => (),
                    SimplifyResult::Replace(g) => gates[i] = *g,
                    SimplifyResult::Solved => {
                        self.solved_gates.insert(i);
                        solved = true;
                    }
                    SimplifyResult::SolvedWitness(w) => {
                        solved = true;
                        if self.is_abi(w) {
                            self.def_gates.insert(i);
                            gates[i] = Opcode::Arithmetic(Expression {
                                mul_terms: Vec::new(),
                                linear_combinations: vec![(FieldElement::one(), w)],
                                q_c: -self.solved[&w],
                            });
                        } else {
                            self.solved_gates.insert(i);
                        }
                    }
                    SimplifyResult::UnsatisfiedConstrain(g) => {
                        return SimplifyResult::UnsatisfiedConstrain(g);
                    }
                }
                if i > 0 {
                    i -= 1;
                } else {
                    break;
                }
                first = false;
            }
        }
        SimplifyResult::Unresolved
    }

    fn simplify_opcode(&mut self, gate: &Opcode, gate_idx: usize, first: bool) -> SimplifyResult {
        if self.solved_gates.contains(&gate_idx) || self.def_gates.contains(&gate_idx) {
            return SimplifyResult::Unresolved;
        }
        match gate {
            Opcode::Arithmetic(expr) => self.simplify_arithmetic(expr, gate_idx, first),
            Opcode::Directive(Directive::Invert { x, result }) => {
                self.simplify_inverse(*x, *result, gate_idx, first)
            }
            Opcode::Directive(Directive::Quotient(quotient)) => {
                self.simplify_quotient(quotient, gate_idx, first)
            }
            Opcode::Directive(Directive::ToLeRadix { a, b, radix }) => {
                self.simplify_radix(a, b.clone(), *radix, gate_idx, first)
            }
            Opcode::BlackBoxFuncCall(gadget) => self.simplify_gadget(gadget, gate_idx, first),
            _ => SimplifyResult::Unresolved,
        }
    }

    fn simplify_gadget(
        &mut self,
        gadget: &BlackBoxFuncCall,
        gate_idx: usize,
        first: bool,
    ) -> SimplifyResult {
        match gadget {
            BlackBoxFuncCall::AND { output, .. } | BlackBoxFuncCall::XOR { output, .. } => {
                self.use_witness(*output, gate_idx, first);
                SimplifyResult::Unresolved
            }
            BlackBoxFuncCall::RANGE { input, .. } => {
                if self.contains(input.witness) {
                    self.use_witness(input.witness, gate_idx, first);
                    let max = BigUint::from_u32(2).unwrap().pow(input.num_bits);
                    if BigUint::from_bytes_be(&self.solved[&input.witness].to_be_bytes()) >= max {
                        return SimplifyResult::UnsatisfiedConstrain(gate_idx);
                    }
                    SimplifyResult::Solved
                } else {
                    SimplifyResult::Unresolved
                }
            }
            _ => {
                for i in gadget.get_inputs_vec() {
                    if self.is_solved(&i.witness) && !self.is_abi(i.witness) {
                        self.defined.insert(i.witness);
                    }
                }
                for i in gadget.get_outputs_vec() {
                    self.use_witness(i, gate_idx, first);
                }
                SimplifyResult::Unresolved
            }
        }
    }

    fn simplify_radix(
        &mut self,
        a: &Expression,
        b: Vec<Witness>,
        radix: u32,
        gate_idx: usize,
        first: bool,
    ) -> SimplifyResult {
        let expr = self.evaluate_arith(a, gate_idx, first);
        if expr != *a {
            SimplifyResult::Replace(Box::new(Opcode::Directive(Directive::ToLeRadix {
                a: expr,
                b,
                radix,
            })))
        } else {
            SimplifyResult::Unresolved
        }
    }

    fn simplify_arithmetic(
        &mut self,
        expression: &Expression,
        gate_idx: usize,
        first: bool,
    ) -> SimplifyResult {
        let expr = self.evaluate_arith(expression, gate_idx, first);

        if expr.is_linear() {
            if expr.linear_combinations.len() == 1 {
                let solved = expr.linear_combinations[0].1;
                if expr.linear_combinations[0].0.is_zero() {
                    return SimplifyResult::UnsatisfiedConstrain(gate_idx);
                }
                return self.insert(solved, -expr.q_c / expr.linear_combinations[0].0, gate_idx);
            } else if expr.linear_combinations.is_empty() {
                if expr.q_c.is_zero() {
                    return SimplifyResult::Solved;
                } else {
                    return SimplifyResult::UnsatisfiedConstrain(gate_idx);
                }
            }
        }
        if expr != *expression {
            SimplifyResult::Replace(Box::new(Opcode::Arithmetic(expr)))
        } else {
            SimplifyResult::Unresolved
        }
    }

    fn simplify_inverse(
        &mut self,
        x: Witness,
        result: Witness,
        gate_idx: usize,
        first: bool,
    ) -> SimplifyResult {
        if result.0 == 44 {}
        self.use_witness(result, gate_idx, first);
        if let Some(f) = self.solved.get(&x) {
            let result_value = if f.is_zero() { FieldElement::zero() } else { f.inverse() };
            self.insert(result, result_value, gate_idx)
        } else {
            if let Some(f) = self.solved.get(&result) {
                if f.is_zero() {
                    return self.insert(x, *f, gate_idx);
                }
            }
            SimplifyResult::Unresolved
        }
    }

    fn solve_fan_in_term_helper(
        term: &(FieldElement, Witness),
        witness_assignments: &BTreeMap<Witness, FieldElement>,
    ) -> Option<FieldElement> {
        let (q_l, w_l) = term;
        if q_l.is_zero() {
            return Some(FieldElement::zero());
        }
        // Check if we have w_l
        let w_l_value = witness_assignments.get(w_l);
        w_l_value.map(|a| *q_l * *a)
    }

    fn solve_mul_term_helper(
        term: &(FieldElement, Witness, Witness),
        witness_assignments: &BTreeMap<Witness, FieldElement>,
    ) -> MulTerm {
        let (q_m, w_l, w_r) = term;
        // Check if these values are in the witness assignments
        let w_l_value = witness_assignments.get(w_l);
        let w_r_value = witness_assignments.get(w_r);

        match (w_l_value, w_r_value) {
            (None, None) => MulTerm::TooManyUnknowns,
            (Some(w_l), Some(w_r)) => MulTerm::Solved(*q_m * *w_l * *w_r),
            (None, Some(w_r)) => MulTerm::OneUnknown(*q_m * *w_r, *w_l),
            (Some(w_l), None) => MulTerm::OneUnknown(*q_m * *w_l, *w_r),
        }
    }

    // Partially evaluate the gate using the known witnesses
    pub fn evaluate_arith(
        &mut self,
        expr: &Expression,
        gate_idx: usize,
        first: bool,
    ) -> Expression {
        let mut result = Expression::default();
        for &(c, w1, w2) in &expr.mul_terms {
            self.use_witness(w1, gate_idx, first);
            self.use_witness(w2, gate_idx, first);
            let mul_result = CircuitSimplifier::solve_mul_term_helper(&(c, w1, w2), &self.solved);
            match mul_result {
                MulTerm::OneUnknown(v, w) => {
                    if !v.is_zero() {
                        result.linear_combinations.push((v, w));
                    }
                }
                MulTerm::TooManyUnknowns => {
                    if !c.is_zero() {
                        result.mul_terms.push((c, w1, w2));
                    }
                }
                MulTerm::Solved(f) => result.q_c += f,
            }
        }
        for &(c, w) in &expr.linear_combinations {
            self.use_witness(w, gate_idx, first);
            if let Some(f) = CircuitSimplifier::solve_fan_in_term_helper(&(c, w), &self.solved) {
                result.q_c += f;
            } else if !c.is_zero() {
                result.linear_combinations.push((c, w));
            }
        }
        result.q_c += expr.q_c;
        result
    }

    fn simplify_quotient(
        &mut self,
        quotient: &QuotientDirective,
        gate_idx: usize,
        first: bool,
    ) -> SimplifyResult {
        // evaluate expressions
        let a_expr = self.evaluate_arith(&quotient.a, gate_idx, first);
        let b_expr = self.evaluate_arith(&quotient.b, gate_idx, first);
        let default = Box::new(Expression::one());
        let pred = quotient.predicate.as_ref().unwrap_or(&default);
        let pred_expr = self.evaluate_arith(pred, gate_idx, first);
        // use witness
        self.use_witness(quotient.q, gate_idx, first);
        self.use_witness(quotient.r, gate_idx, first);
        if a_expr.is_const() && b_expr.is_const() && pred_expr.is_const() {
            let val_a = a_expr.q_c;
            let val_b = b_expr.q_c;
            //
            let int_a = BigUint::from_bytes_be(&val_a.to_be_bytes());
            let int_b = BigUint::from_bytes_be(&val_b.to_be_bytes());
            let pred_value = pred_expr.q_c;
            let (int_r, int_q) = if pred_value.is_zero() {
                (BigUint::zero(), BigUint::zero())
            } else {
                (&int_a % &int_b, &int_a / &int_b)
            };
            let r1 = self.insert(
                quotient.q,
                FieldElement::from_be_bytes_reduce(&int_q.to_bytes_be()),
                gate_idx,
            );
            let r2 = self.insert(
                quotient.r,
                FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()),
                gate_idx,
            );
            if r1 == SimplifyResult::UnsatisfiedConstrain(gate_idx)
                || r2 == SimplifyResult::UnsatisfiedConstrain(gate_idx)
            {
                SimplifyResult::UnsatisfiedConstrain(gate_idx)
            } else {
                SimplifyResult::Solved
            }
        } else if a_expr.is_degree_one_univariate()
            && b_expr.is_const()
            && pred_expr.is_const()
            && self.contains(quotient.q)
            && self.contains(quotient.r)
        {
            let a_witness = a_expr.linear_combinations[0].1;
            self.insert(
                a_witness,
                b_expr.q_c * self.solved[&quotient.q] + self.solved[&quotient.r],
                gate_idx,
            )
        } else if a_expr.is_zero() || pred_expr.is_zero() {
            let r1 = self.insert(quotient.q, FieldElement::zero(), gate_idx);
            let r2 = self.insert(quotient.r, FieldElement::zero(), gate_idx);
            if r1 == SimplifyResult::UnsatisfiedConstrain(gate_idx)
                || r2 == SimplifyResult::UnsatisfiedConstrain(gate_idx)
            {
                SimplifyResult::UnsatisfiedConstrain(gate_idx)
            } else {
                SimplifyResult::Solved
            }
        } else if a_expr != quotient.a || b_expr != quotient.b {
            let new_quotient = QuotientDirective {
                a: a_expr,
                b: b_expr,
                q: quotient.q,
                r: quotient.r,
                predicate: quotient.predicate.clone(),
            };
            SimplifyResult::Replace(Box::new(Opcode::Directive(Directive::Quotient(new_quotient))))
        } else {
            SimplifyResult::Unresolved
        }
    }
}

#[cfg(test)]
mod test {
    use acir::{
        circuit::{Circuit, Opcode},
        native_types::{Expression, Witness},
        FieldElement,
    };

    use crate::compiler::{optimizers::Simplifier, transformers::FallbackTransformer};

    #[test]
    fn simplify_test() {
        let a = Witness(0);
        let b = Witness(1);
        let c = Witness(2);
        let d = Witness(3);

        let one = FieldElement::one();
        // b = c * d ;
        let gate_b = Expression {
            mul_terms: vec![(one, b, c)],
            linear_combinations: vec![(-one, a)],
            q_c: FieldElement::zero(),
        };
        // d = 3;
        let gate_d = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(one, d)],
            q_c: FieldElement::from(-3_i128),
        };
        // a = 0;
        let gate_a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(one, a)],
            q_c: FieldElement::zero(),
        };
        let mut simplifier = Simplifier::new(1);
        let mut circuit = vec![
            Opcode::Arithmetic(gate_a),
            Opcode::Arithmetic(gate_b),
            Opcode::Arithmetic(gate_d),
        ];
        simplifier.simplify(&mut circuit);
        assert_eq!(circuit.len(), 3);
        assert_eq!(simplifier.solved_gates.len(), 1);
        let support_all = |_opcode: &Opcode| true;
        let mut acir = Circuit::default();
        acir.opcodes = circuit;
        let acir = FallbackTransformer::transform(acir, support_all, &simplifier).unwrap();
        assert_eq!(acir.opcodes.len(), 2);
    }
}
