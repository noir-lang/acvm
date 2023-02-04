use std::collections::BTreeMap;

use acir::{circuit::directives::Directive, native_types::Witness, FieldElement};
use num_bigint::BigUint;
use num_traits::{One, Zero};

use crate::OpcodeResolutionError;

use super::{get_value, insert_value, witness_to_value};

pub fn solve_directives(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    directive: &Directive,
) -> Result<(), OpcodeResolutionError> {
    match directive {
        Directive::Invert { x, result } => {
            let val = witness_to_value(initial_witness, *x)?;
            let inverse = val.inverse();
            initial_witness.insert(*result, inverse);
            Ok(())
        }
        Directive::Quotient {
            a,
            b,
            q,
            r,
            predicate,
        } => {
            let val_a = get_value(a, initial_witness)?;
            let val_b = get_value(b, initial_witness)?;

            let int_a = BigUint::from_bytes_be(&val_a.to_be_bytes());
            let int_b = BigUint::from_bytes_be(&val_b.to_be_bytes());

            // If the predicate is `None`, then we simply return the value 1
            // If the predicate is `Some` but we cannot find a value, then we return unresolved
            let pred_value = match predicate {
                Some(pred) => get_value(pred, initial_witness)?,
                None => FieldElement::one(),
            };

            let (int_r, int_q) = if pred_value.is_zero() {
                (BigUint::zero(), BigUint::zero())
            } else {
                (&int_a % &int_b, &int_a / &int_b)
            };

            initial_witness.insert(*q, FieldElement::from_be_bytes_reduce(&int_q.to_bytes_be()));
            initial_witness.insert(*r, FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()));

            Ok(())
        }
        Directive::Truncate { a, b, c, bit_size } => {
            let val_a = get_value(a, initial_witness)?;

            let pow: BigUint = BigUint::one() << bit_size;

            let int_a = BigUint::from_bytes_be(&val_a.to_be_bytes());
            let int_b: BigUint = &int_a % &pow;
            let int_c: BigUint = (&int_a - &int_b) / &pow;

            initial_witness.insert(*b, FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()));
            initial_witness.insert(*c, FieldElement::from_be_bytes_reduce(&int_c.to_bytes_be()));

            Ok(())
        }
        Directive::ToRadix {
            a,
            b,
            radix,
            is_little_endian,
        } => {
            let value_a = get_value(a, initial_witness)?;

            let big_integer = BigUint::from_bytes_be(&value_a.to_be_bytes());

            // Decompose the integer into its radix digits
            let decomposed_integer = if *is_little_endian {
                big_integer.to_radix_le(*radix)
            } else {
                big_integer.to_radix_be(*radix)
            };

            to_radix_outcome(b, decomposed_integer, initial_witness)
        }
        Directive::OddRange { a, b, r, bit_size } => {
            let val_a = witness_to_value(initial_witness, *a)?;

            let int_a = BigUint::from_bytes_be(&val_a.to_be_bytes());
            let pow: BigUint = BigUint::one() << (bit_size - 1);
            if int_a >= (&pow << 1) {
                return Err(OpcodeResolutionError::UnsatisfiedConstrain);
            }

            let bb = &int_a & &pow;
            let int_r = &int_a - &bb;
            let int_b = &bb >> (bit_size - 1);

            initial_witness.insert(*b, FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()));
            initial_witness.insert(*r, FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()));

            Ok(())
        }
    }
}

fn to_radix_outcome(
    witnesses: &Vec<Witness>,
    decomposed_integer: Vec<u8>,
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
) -> Result<(), OpcodeResolutionError> {
    if witnesses.len() < decomposed_integer.len() {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain);
    }

    for (i, witness) in witnesses.into_iter().enumerate() {
        // Fetch the `i'th` digit from the decomposed integer list
        // and convert it to a field element.
        // If it is not available, which can happen when the decomposed integer
        // list is shorter than the witness list, we return 0.
        let value = match decomposed_integer.get(i) {
            Some(digit) => FieldElement::from_be_bytes_reduce(&[*digit]),
            None => FieldElement::zero(),
        };

        insert_value(witness, value, initial_witness)?
    }

    Ok(())
}
