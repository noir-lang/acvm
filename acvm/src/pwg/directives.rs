use std::{cmp::Ordering, collections::BTreeMap};

use acir::{
    circuit::directives::{Directive, LogInfo},
    native_types::Witness,
    FieldElement,
};
use num_bigint::BigUint;
use num_traits::{One, Zero};

use crate::OpcodeResolutionError;

use super::{get_value, sorting::route, witness_to_value};

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

            insert_witness(
                *q,
                FieldElement::from_be_bytes_reduce(&int_q.to_bytes_be()),
                initial_witness,
            )?;
            insert_witness(
                *r,
                FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()),
                initial_witness,
            )?;

            Ok(())
        }
        Directive::Truncate { a, b, c, bit_size } => {
            let val_a = get_value(a, initial_witness)?;

            let pow: BigUint = BigUint::one() << bit_size;

            let int_a = BigUint::from_bytes_be(&val_a.to_be_bytes());
            let int_b: BigUint = &int_a % &pow;
            let int_c: BigUint = (&int_a - &int_b) / &pow;

            insert_witness(
                *b,
                FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()),
                initial_witness,
            )?;
            insert_witness(
                *c,
                FieldElement::from_be_bytes_reduce(&int_c.to_bytes_be()),
                initial_witness,
            )?;

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
                insert_witness(b[i], v, initial_witness)?;
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

            insert_witness(
                *b,
                FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()),
                initial_witness,
            )?;
            insert_witness(
                *r,
                FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()),
                initial_witness,
            )?;

            Ok(())
        }
        Directive::PermutationSort {
            inputs: a,
            tuple,
            bits,
            sort_by,
        } => {
            let mut val_a = Vec::new();
            let mut base = Vec::new();
            for (i, element) in a.iter().enumerate() {
                assert_eq!(element.len(), *tuple as usize);
                let mut element_val = Vec::with_capacity(*tuple as usize + 1);
                for e in element {
                    element_val.push(get_value(e, initial_witness)?);
                }
                let field_i = FieldElement::from(i as i128);
                element_val.push(field_i);
                base.push(field_i);
                val_a.push(element_val);
            }
            val_a.sort_by(|a, b| {
                for i in sort_by {
                    let int_a = BigUint::from_bytes_be(&a[*i as usize].to_be_bytes());
                    let int_b = BigUint::from_bytes_be(&b[*i as usize].to_be_bytes());
                    let cmp = int_a.cmp(&int_b);
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                Ordering::Equal
            });
            let b = val_a.iter().map(|a| *a.last().unwrap()).collect();
            let control = route(base, b);
            for (w, value) in bits.iter().zip(control) {
                let value = if value {
                    FieldElement::one()
                } else {
                    FieldElement::zero()
                };
                insert_witness(*w, value, initial_witness)?;
            }
            Ok(())
        }
        Directive::Log(info) => {
            let witnesses = match info {
                LogInfo::FinalizedOutput(output_string) => {
                    println!("{output_string}");
                    return Ok(());
                }
                LogInfo::WitnessOutput(witnesses) => witnesses,
            };

            if witnesses.len() == 1 {
                let witness = &witnesses[0];
                let log_value = witness_to_value(initial_witness, *witness)?;
                println!("{}", log_value.to_hex());

                return Ok(());
            }

            // If multiple witnesses are to be fetched for a log directive,
            // it assumed that an array is meant to be printed to standard output
            //
            // Collect all field element values corresponding to the given witness indices
            // and convert them to hex strings.
            let mut elements_as_hex = Vec::with_capacity(witnesses.len());
            for witness in witnesses {
                let element = witness_to_value(initial_witness, *witness)?;
                elements_as_hex.push(element.to_hex());
            }

            // Join all of the hex strings using a comma
            let comma_separated_elements = elements_as_hex.join(",");

            let output_witnesses_string = "[".to_owned() + &comma_separated_elements + "]";

            println!("{output_witnesses_string}");

            Ok(())
        }
        Directive::Log(info) => {
            let witnesses = match info {
                LogInfo::FinalizedOutput(output_string) => {
                    println!("{output_string}");
                    return Ok(());
                }
                LogInfo::WitnessOutput(witnesses) => witnesses,
            };

            if witnesses.len() == 1 {
                let witness = &witnesses[0];
                let log_value = witness_to_value(initial_witness, *witness)?;
                println!("{}", log_value.to_hex());

                return Ok(());
            }

            // If multiple witnesses are to be fetched for a log directive,
            // it assumed that an array is meant to be printed to standard output
            //
            // Collect all field element values corresponding to the given witness indices
            // and convert them to hex strings.
            let mut elements_as_hex = Vec::with_capacity(witnesses.len());
            for witness in witnesses {
                let element = witness_to_value(initial_witness, *witness)?;
                elements_as_hex.push(element.to_hex());
            }

            // Join all of the hex strings using a comma
            let comma_separated_elements = elements_as_hex.join(",");

            let output_witnesses_string = "[".to_owned() + &comma_separated_elements + "]";

            println!("{output_witnesses_string}");

            Ok(())
        }
    }
}

fn insert_witness(
    w: Witness,
    value: FieldElement,
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
) -> Result<(), OpcodeResolutionError> {
    match initial_witness.entry(w) {
        std::collections::btree_map::Entry::Vacant(e) => {
            e.insert(value);
        }
        std::collections::btree_map::Entry::Occupied(e) => {
            if e.get() != &value {
                return Err(OpcodeResolutionError::UnsatisfiedConstrain);
            }
        }
    }
    Ok(())
}
