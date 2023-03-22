use std::{cmp::Ordering, collections::BTreeMap};

use acir::{
    circuit::directives::{Directive, LogOutputInfo, SolvedLog, SolvedLogOutputInfo},
    native_types::Witness,
    FieldElement,
};
use num_bigint::BigUint;
use num_traits::Zero;

use crate::OpcodeResolutionError;

use super::{get_value, insert_value, sorting::route, witness_to_value};

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
        Directive::Quotient { a, b, q, r, predicate } => {
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
        Directive::ToLeRadix { a, b, radix } => {
            let value_a = get_value(a, initial_witness)?;
            let big_integer = BigUint::from_bytes_be(&value_a.to_be_bytes());

            // Decompose the integer into its radix digits in little endian form.
            let decomposed_integer = big_integer.to_radix_le(*radix);

            if b.len() < decomposed_integer.len() {
                return Err(OpcodeResolutionError::UnsatisfiedConstrain);
            }

            for (i, witness) in b.iter().enumerate() {
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
        Directive::PermutationSort { inputs: a, tuple, bits, sort_by } => {
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
                let value = if value { FieldElement::one() } else { FieldElement::zero() };
                insert_witness(*w, value, initial_witness)?;
            }
            Ok(())
        }
        Directive::Log { .. } => {
            // let witnesses = match output_info {
            //     LogOutputInfo::FinalizedOutput(final_string) => {
            //         return Ok(Some(SolvedLog {
            //             trace_label: trace_label.clone(),
            //             output_info: SolvedLogOutputInfo::FinalizedOutput(final_string.clone()),
            //         }))
            //     }
            //     LogOutputInfo::WitnessOutput(witnesses) => witnesses,
            // };

            // let mut elements = Vec::with_capacity(witnesses.len());
            // for witness in witnesses {
            //     let element = witness_to_value(initial_witness, *witness)?;
            //     elements.push(*element);
            // }

            // let solved_log = SolvedLog {
            //     trace_label: trace_label.clone(),
            //     output_info: SolvedLogOutputInfo::WitnessValues(elements),
            // };

            // Ok(Some(solved_log))
            Ok(())
        }
    }
}

pub fn insert_witness(
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
