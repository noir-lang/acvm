use std::cmp::Ordering;

use acir::{
    circuit::directives::{Directive, LogInfo, QuotientDirective},
    native_types::WitnessMap,
    FieldElement,
};
use num_bigint::BigUint;
use num_traits::Zero;

use crate::{pwg::OpcodeResolution, OpcodeResolutionError};

use super::{get_value, insert_value, sorting::route, witness_to_value};

/// Attempts to solve the [`Directive`] opcode `directive`.
/// If successful, `initial_witness` will be mutated to contain the new witness assignment.
///
/// Returns `Ok(OpcodeResolution)` to signal whether the directive was successful solved.
///
/// Returns `Err(OpcodeResolutionError)` if a circuit constraint is unsatisfied.
pub fn solve_directives(
    initial_witness: &mut WitnessMap,
    directive: &Directive,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    match solve_directives_internal(initial_witness, directive) {
        Ok(_) => Ok(OpcodeResolution::Solved),
        Err(OpcodeResolutionError::OpcodeNotSolvable(unsolved)) => {
            Ok(OpcodeResolution::Stalled(unsolved))
        }
        Err(err) => Err(err),
    }
}

fn solve_directives_internal(
    initial_witness: &mut WitnessMap,
    directive: &Directive,
) -> Result<(), OpcodeResolutionError> {
    match directive {
        Directive::Invert { x, result } => {
            let val = witness_to_value(initial_witness, *x)?;
            let inverse = val.inverse();
            initial_witness.insert(*result, inverse);
            Ok(())
        }
        Directive::Quotient(QuotientDirective { a, b, q, r, predicate }) => {
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

            insert_value(
                q,
                FieldElement::from_be_bytes_reduce(&int_q.to_bytes_be()),
                initial_witness,
            )?;
            insert_value(
                r,
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
                insert_value(w, value, initial_witness)?;
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
                println!("{}", format_field_string(*log_value));
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
                elements_as_hex.push(format_field_string(*element));
            }

            // Join all of the hex strings using a comma
            let comma_separated_elements = elements_as_hex.join(", ");

            let output_witnesses_string = "[".to_owned() + &comma_separated_elements + "]";

            println!("{output_witnesses_string}");

            Ok(())
        }
    }
}

/// This trims any leading zeroes.
/// A singular '0' will be prepended as well if the trimmed string has an odd length.
/// A hex string's length needs to be even to decode into bytes, as two digits correspond to
/// one byte.
fn format_field_string(field: FieldElement) -> String {
    let mut trimmed_field = field.to_hex().trim_start_matches('0').to_owned();
    if trimmed_field.len() % 2 != 0 {
        trimmed_field = "0".to_owned() + &trimmed_field
    }
    "0x".to_owned() + &trimmed_field
}
