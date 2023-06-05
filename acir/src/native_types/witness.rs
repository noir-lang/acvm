use std::ops::{Add, Mul, Sub};

use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

use super::Expression;

// Witness might be a misnomer. This is an index that represents the position a witness will take
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct Witness(pub u32);

impl Witness {
    pub fn new(witness_index: u32) -> Witness {
        Witness(witness_index)
    }
    pub fn witness_index(&self) -> u32 {
        self.0
    }
    pub fn as_usize(&self) -> usize {
        // This is safe as long as the architecture is 32bits minimum
        self.0 as usize
    }

    pub const fn can_defer_constraint(&self) -> bool {
        true
    }
}

impl From<u32> for Witness {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

// Operators

impl Add for Witness {
    type Output = Expression;
    fn add(self, rhs: Witness) -> Self::Output {
        if self == rhs {
            Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![(FieldElement::from(2_u128), self)],
                q_c: FieldElement::zero(),
            }
        } else {
            let linear_combinations = if self < rhs {
                vec![(FieldElement::one(), self), (FieldElement::one(), rhs)]
            } else {
                vec![(FieldElement::one(), rhs), (FieldElement::one(), self)]
            };

            Expression { mul_terms: Vec::new(), linear_combinations, q_c: FieldElement::zero() }
        }
    }
}

impl Sub for Witness {
    type Output = Expression;
    fn sub(self, rhs: Witness) -> Self::Output {
        if self == rhs {
            Expression::zero()
        } else {
            let linear_combinations = if self < rhs {
                vec![(FieldElement::one(), self), (-FieldElement::one(), rhs)]
            } else {
                vec![(-FieldElement::one(), rhs), (FieldElement::one(), self)]
            };

            Expression { mul_terms: Vec::new(), q_c: FieldElement::zero(), linear_combinations }
        }
    }
}

impl Mul for Witness {
    type Output = Expression;
    fn mul(self, rhs: Witness) -> Self::Output {
        if self == rhs {
            Expression {
                mul_terms: vec![(FieldElement::one(), self, self)],
                linear_combinations: vec![],
                q_c: FieldElement::zero(),
            }
        } else {
            let mul_terms = if self < rhs {
                vec![(FieldElement::one(), self, rhs)]
            } else {
                vec![(FieldElement::one(), rhs, self)]
            };

            Expression { mul_terms, linear_combinations: Vec::new(), q_c: FieldElement::zero() }
        }
    }
}
