use std::collections::BTreeMap;

use acir::{
    circuit::directives::{Directive, LogInfo},
    native_types::Witness,
    FieldElement,
};
use num_bigint::BigUint;
use num_traits::{One, Zero};

use crate::{OpcodeNotSolvable, OpcodeResolutionError};

use super::{get_value, witness_to_value};

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
        Directive::ToRadix { a, b, radix } => {
            let val_a = get_value(a, initial_witness)?;

            let a_big = BigUint::from_bytes_be(&val_a.to_be_bytes());
            let a_dec = a_big.to_radix_le(*radix);
            if b.len() < a_dec.len() {
                return Err(OpcodeResolutionError::UnsatisfiedConstrain);
            }
            for i in 0..b.len() {
                let v = if i < a_dec.len() {
                    FieldElement::from_be_bytes_reduce(&[a_dec[i]])
                } else {
                    FieldElement::zero()
                };
                match initial_witness.entry(b[i]) {
                    std::collections::btree_map::Entry::Vacant(e) => {
                        e.insert(v);
                    }
                    std::collections::btree_map::Entry::Occupied(e) => {
                        if e.get() != &v {
                            return Err(OpcodeResolutionError::UnsatisfiedConstrain);
                        }
                    }
                }
            }

            Ok(())
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
        Directive::Log(info) => {
            match info {
                LogInfo::FinalizedOutput(output_string) => println!("{}", output_string),
                LogInfo::WitnessOutput(witnesses) => {
                    if witnesses.len() == 1 {
                        match initial_witness.entry(witnesses[0]) {
                            std::collections::btree_map::Entry::Vacant(_) => {
                                unreachable!("log entry does must have a witness");
                            }
                            std::collections::btree_map::Entry::Occupied(e) => {
                                println!("{}", e.get().to_hex());
                            }
                        }
                    } else {
                        // If multiple witnesses are to be fetched for a log directive,
                        // it assumed that an array is meant to be printed to standard output
                        let mut output_witnesses_string = "".to_owned();
                        output_witnesses_string.push_str("[");
                        let mut iter = witnesses.iter().peekable();
                        while let Some(w) = iter.next() {
                            let elem = match initial_witness.entry(*w) {
                                std::collections::btree_map::Entry::Vacant(_) => {
                                    return Err(OpcodeResolutionError::OpcodeNotSolvable(
                                        OpcodeNotSolvable::MissingAssignment(w.0),
                                    ))
                                }
                                std::collections::btree_map::Entry::Occupied(e) => e.get().clone(),
                            };
                            if iter.peek().is_none() {
                                output_witnesses_string.push_str(&format!("{}", elem.to_hex()));
                            } else {
                                output_witnesses_string.push_str(&format!("{}, ", elem.to_hex()));
                            }
                        }
                        output_witnesses_string.push_str("]");
                        println!("{}", output_witnesses_string);
                    }
                }
            }
            Ok(())
        }
    }
}
