use crate::native_types::Witness;
use acir_field::FieldElement;
use std::ops::{Add, Mul, Neg, Sub};

use super::Expression;

// Negation

impl Neg for &Expression {
    type Output = Expression;
    fn neg(self) -> Self::Output {
        // XXX(med) : Implement an efficient way to do this

        let mul_terms: Vec<_> =
            self.mul_terms.iter().map(|(q_m, w_l, w_r)| (-*q_m, *w_l, *w_r)).collect();

        let linear_combinations: Vec<_> =
            self.linear_combinations.iter().map(|(q_k, w_k)| (-*q_k, *w_k)).collect();
        let q_c = -self.q_c;

        Expression { mul_terms, linear_combinations, q_c }
    }
}

// FieldElement

impl Add<FieldElement> for Expression {
    type Output = Expression;
    fn add(self, rhs: FieldElement) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c + rhs;

        Expression { mul_terms: self.mul_terms, q_c, linear_combinations: self.linear_combinations }
    }
}

impl Add<Expression> for FieldElement {
    type Output = Expression;
    #[inline]
    fn add(self, rhs: Expression) -> Self::Output {
        rhs + self
    }
}

impl Sub<FieldElement> for Expression {
    type Output = Expression;
    fn sub(self, rhs: FieldElement) -> Self::Output {
        // Increase the constant
        let q_c = self.q_c - rhs;

        Expression { mul_terms: self.mul_terms, q_c, linear_combinations: self.linear_combinations }
    }
}

impl Sub<Expression> for FieldElement {
    type Output = Expression;
    #[inline]
    fn sub(self, rhs: Expression) -> Self::Output {
        rhs - self
    }
}

impl Mul<FieldElement> for &Expression {
    type Output = Expression;
    fn mul(self, rhs: FieldElement) -> Self::Output {
        // Scale the mul terms
        let mul_terms: Vec<_> =
            self.mul_terms.iter().map(|(q_m, w_l, w_r)| (*q_m * rhs, *w_l, *w_r)).collect();

        // Scale the linear combinations terms
        let lin_combinations: Vec<_> =
            self.linear_combinations.iter().map(|(q_l, w_l)| (*q_l * rhs, *w_l)).collect();

        // Scale the constant
        let q_c = self.q_c * rhs;

        Expression { mul_terms, q_c, linear_combinations: lin_combinations }
    }
}

impl Mul<&Expression> for FieldElement {
    type Output = Expression;
    #[inline]
    fn mul(self, rhs: &Expression) -> Self::Output {
        rhs * self
    }
}

// Witness

impl Add<Witness> for &Expression {
    type Output = Expression;
    fn add(self, rhs: Witness) -> Expression {
        self + &Expression::from(rhs)
    }
}

impl Add<&Expression> for Witness {
    type Output = Expression;
    #[inline]
    fn add(self, rhs: &Expression) -> Expression {
        rhs + self
    }
}

impl Sub<Witness> for &Expression {
    type Output = Expression;
    fn sub(self, rhs: Witness) -> Expression {
        self - &Expression::from(rhs)
    }
}

impl Sub<&Expression> for Witness {
    type Output = Expression;
    #[inline]
    fn sub(self, rhs: &Expression) -> Expression {
        rhs - self
    }
}

// Mul<Witness> is not implemented as this could result in degree 3 terms.

// Expression

impl Add<&Expression> for &Expression {
    type Output = Expression;
    fn add(self, rhs: &Expression) -> Expression {
        // XXX(med) : Implement an efficient way to do this

        let mul_terms: Vec<_> =
            self.mul_terms.iter().cloned().chain(rhs.mul_terms.iter().cloned()).collect();

        let linear_combinations: Vec<_> = self
            .linear_combinations
            .iter()
            .cloned()
            .chain(rhs.linear_combinations.iter().cloned())
            .collect();
        let q_c = self.q_c + rhs.q_c;

        Expression { mul_terms, linear_combinations, q_c }
    }
}

impl Sub<&Expression> for &Expression {
    type Output = Expression;
    fn sub(self, rhs: &Expression) -> Expression {
        self + &-rhs
    }
}

// Mul<Expression> is not implemented as this could result in degree 3+ terms.
