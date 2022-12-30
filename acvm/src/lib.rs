// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub mod compiler;
pub mod pwg;

use crate::pwg::{arithmetic::ArithmeticSolver, logic::LogicSolver, witness_to_value};
use acir::{
    circuit::{directives::Directive, opcodes::BlackBoxFuncCall, Circuit, Opcode},
    native_types::{Expression, Witness},
    BlackBoxFunc,
};
use blake2::digest::FixedOutput;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::collections::BTreeMap;
use thiserror::Error;

// re-export acir
pub use acir;
pub use acir::FieldElement;

// This enum represents the different cases in which an
// opcode can be unsolvable.
// The most common being that one of its input has not been
// assigned a value.
//
// TODO: ExpressionHasTooManyUnknowns is specific for arithmetic expressions
// TODO: we could have a error enum for arithmetic failure cases in that module
// TODO that can be converted into an OpcodeNotSolvable or OpcodeResolutionError enum
#[derive(PartialEq, Eq, Debug, Error)]
pub enum OpcodeNotSolvable {
    #[error("missing assignment for witness index {0}")]
    MissingAssignment(u32),
    #[error("expression has too many unknowns {0}")]
    ExpressionHasTooManyUnknowns(Expression),
}

#[derive(PartialEq, Eq, Debug, Error)]
pub enum OpcodeResolutionError {
    #[error("cannot solve opcode: {0}")]
    OpcodeNotSolvable(OpcodeNotSolvable),
    #[error("backend does not currently support the {0} opcode. ACVM does not currently have a fallback for this opcode.")]
    UnsupportedBlackBoxFunc(BlackBoxFunc),
    #[error("could not satisfy all constraints")]
    UnsatisfiedConstrain,
    #[error("unexpected opcode, expected {0}, but got {1}")]
    UnexpectedOpcode(&'static str, BlackBoxFunc),
    #[error("expected {0} inputs for function {1}, but got {2}")]
    IncorrectNumFunctionArguments(usize, BlackBoxFunc, usize),
}

pub trait Backend: SmartContract + ProofSystemCompiler + PartialWitnessGenerator {}

/// This component will generate the backend specific output for
/// each OPCODE.
/// Returns an Error if the backend does not support that OPCODE
pub trait PartialWitnessGenerator {
    fn solve(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gates: Vec<Opcode>,
    ) -> Result<(), OpcodeResolutionError> {
        if gates.is_empty() {
            return Ok(());
        }
        let mut unsolved_opcodes: Vec<Opcode> = Vec::new();

        for gate in gates.into_iter() {
            let resolution = match &gate {
                Opcode::Arithmetic(expr) => ArithmeticSolver::solve(initial_witness, expr),
                Opcode::BlackBoxFuncCall(bb_func) => {
                    Self::solve_blackbox_function_call(initial_witness, bb_func)
                }
                Opcode::Directive(directive) => Self::solve_directives(initial_witness, directive),
            };

            match resolution {
                Ok(_) => {
                    // We do nothing in the happy case
                }
                Err(OpcodeResolutionError::OpcodeNotSolvable(_)) => {
                    // For opcode not solvable errors, we push those opcodes to the back as
                    // it could be because the opcodes are out of order, ie this assignment
                    // relies on a later gate's results
                    unsolved_opcodes.push(gate);
                }
                Err(err) => return Err(err),
            }
        }
        self.solve(initial_witness, unsolved_opcodes)
    }

    fn solve_blackbox_function_call(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<(), OpcodeResolutionError>;

    fn solve_range_opcode(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<(), OpcodeResolutionError> {
        // TODO: this consistency check can be moved to a general function
        let defined_input_size = BlackBoxFunc::RANGE
            .definition()
            .input_size
            .fixed_size()
            .expect("infallible: input for range gate is fixed");

        let num_arguments = func_call.inputs.len();
        if num_arguments != defined_input_size as usize {
            return Err(OpcodeResolutionError::IncorrectNumFunctionArguments(
                defined_input_size as usize,
                BlackBoxFunc::RANGE,
                num_arguments,
            ));
        }

        // For the range constraint, we know that the input size should be one
        assert_eq!(defined_input_size, 1);

        let input = func_call
            .inputs
            .first()
            .expect("infallible: checked that input size is 1");

        let w_value = witness_to_value(initial_witness, input.witness)?;

        if w_value.num_bits() > input.num_bits {
            return Err(OpcodeResolutionError::UnsatisfiedConstrain);
        }

        Ok(())
    }

    fn solve_logic_opcode(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<(), OpcodeResolutionError> {
        match func_call.name {
            BlackBoxFunc::AND => LogicSolver::solve_and_gate(initial_witness, &func_call),
            BlackBoxFunc::XOR => LogicSolver::solve_xor_gate(initial_witness, &func_call),
            _ => Err(OpcodeResolutionError::UnexpectedOpcode(
                "logic opcode",
                func_call.name,
            )),
        }
    }

    // Check if all of the inputs to the function have assignments
    // Returns true if all of the inputs have been assigned
    fn all_func_inputs_assigned(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> bool {
        // This call to .any returns true, if any of the witnesses do not have assignments
        // We then use `!`, so it returns false if any of the witnesses do not have assignments
        !func_call
            .inputs
            .iter()
            .any(|input| !initial_witness.contains_key(&input.witness))
    }

    fn get_value(
        expr: &Expression,
        initial_witness: &BTreeMap<Witness, FieldElement>,
    ) -> Result<FieldElement, OpcodeResolutionError> {
        let mut result = expr.q_c;

        for term in &expr.linear_combinations {
            let coefficient = term.0;
            let variable = term.1;

            // Get the value assigned to that variable
            let assignment = *witness_to_value(initial_witness, variable)?;

            result += coefficient * assignment;
        }

        for term in &expr.mul_terms {
            let coefficient = term.0;
            let lhs_variable = term.1;
            let rhs_variable = term.2;

            // Get the values assigned to those variables
            let lhs_assignment = *witness_to_value(initial_witness, lhs_variable)?;
            let rhs_assignment = *witness_to_value(initial_witness, rhs_variable)?;

            result += coefficient * lhs_assignment * rhs_assignment;
        }

        Ok(result)
    }

    fn solve_directives(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        directive: &Directive,
    ) -> Result<(), OpcodeResolutionError> {
        match directive {
            Directive::Invert { x, result } => {
                let val = witness_to_value(initial_witness, *x)?;
                let inverse = val.inverse();
                initial_witness.insert(*result, inverse);
                return Ok(());
            }
            Directive::Quotient {
                a,
                b,
                q,
                r,
                predicate,
            } => {
                let val_a = Self::get_value(a, initial_witness)?;
                let val_b = Self::get_value(b, initial_witness)?;

                let int_a = BigUint::from_bytes_be(&val_a.to_be_bytes());
                let int_b = BigUint::from_bytes_be(&val_b.to_be_bytes());

                // If the predicate is `None`, then we simply return the value 1
                // If the predicate is `Some` but we cannot find a value, then we return unresolved
                let pred_value = match predicate {
                    Some(pred) => Self::get_value(pred, initial_witness)?,
                    None => FieldElement::one(),
                };

                let (int_r, int_q) = if pred_value.is_zero() {
                    (BigUint::zero(), BigUint::zero())
                } else {
                    (&int_a % &int_b, &int_a / &int_b)
                };

                initial_witness
                    .insert(*q, FieldElement::from_be_bytes_reduce(&int_q.to_bytes_be()));
                initial_witness
                    .insert(*r, FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()));

                Ok(())
            }
            Directive::Truncate { a, b, c, bit_size } => {
                let val_a = witness_to_value(initial_witness, *a)?;

                let pow: BigUint = BigUint::one() << bit_size;

                let int_a = BigUint::from_bytes_be(&val_a.to_be_bytes());
                let int_b: BigUint = &int_a % &pow;
                let int_c: BigUint = (&int_a - &int_b) / &pow;

                initial_witness
                    .insert(*b, FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()));
                initial_witness
                    .insert(*c, FieldElement::from_be_bytes_reduce(&int_c.to_bytes_be()));

                Ok(())
            }
            Directive::ToBits { a, b, bit_size } => {
                let val_a = Self::get_value(a, initial_witness)?;

                let a_big = BigUint::from_bytes_be(&val_a.to_be_bytes());
                for j in 0..*bit_size as usize {
                    let v = if a_big.bit(j as u64) {
                        FieldElement::one()
                    } else {
                        FieldElement::zero()
                    };

                    match initial_witness.entry(b[j]) {
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
            Directive::ToBytes { a, b, byte_size } => {
                let val_a = Self::get_value(a, initial_witness)?;

                let mut a_bytes = val_a.to_be_bytes();
                a_bytes.reverse();

                for i in 0..*byte_size as usize {
                    let v = FieldElement::from_be_bytes_reduce(&[a_bytes[i]]);
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

                initial_witness
                    .insert(*b, FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()));
                initial_witness
                    .insert(*r, FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()));

                Ok(())
            }
        }
    }
}

pub trait SmartContract {
    // Takes a verification  key and produces a smart contract
    // The platform indicator allows a backend to support multiple smart contract platforms
    //
    // fn verification_key(&self, platform: u8, vk: &[u8]) -> &[u8] {
    //     todo!("currently the backend is not configured to use this.")
    // }

    /// Takes an ACIR circuit, the number of witnesses and the number of public inputs
    /// Then returns an Ethereum smart contract
    ///
    /// XXX: This will be deprecated in future releases for the above method.
    /// This deprecation may happen in two stages:
    /// The first stage will remove `num_witnesses` and `num_public_inputs` parameters.
    /// If we cannot avoid `num_witnesses`, it can be added into the Circuit struct.
    fn eth_contract_from_cs(&self, circuit: Circuit) -> String;
}

pub trait ProofSystemCompiler {
    /// The NPC language that this proof system directly accepts.
    /// It is possible for ACVM to transpile to different languages, however it is advised to create a new backend
    /// as this in most cases will be inefficient. For this reason, we want to throw a hard error
    /// if the language and proof system does not line up.
    fn np_language(&self) -> Language;
    // Returns true if the backend supports the selected blackbox function
    fn blackbox_function_supported(&self, opcode: &BlackBoxFunc) -> bool;

    /// Creates a Proof given the circuit description and the witness values.
    /// It is important to note that the intermediate witnesses for blackbox functions will not generated
    /// This is the responsibility of the proof system.
    ///
    /// See `SmartContract` regarding the removal of `num_witnesses` and `num_public_inputs`
    fn prove_with_meta(
        &self,
        circuit: Circuit,
        witness_values: BTreeMap<Witness, FieldElement>,
    ) -> Vec<u8>;

    /// Verifies a Proof, given the circuit description.
    ///
    /// XXX: This will be changed in the future to accept a VerifierKey.
    /// At the moment, the Aztec backend API only accepts a constraint system,
    /// which is why this is here.
    ///
    /// See `SmartContract` regarding the removal of `num_witnesses` and `num_public_inputs`
    fn verify_from_cs(
        &self,
        proof: &[u8],
        public_input: Vec<FieldElement>,
        circuit: Circuit,
    ) -> bool;

    fn get_exact_circuit_size(&self, circuit: Circuit) -> u32;
}

/// Supported NP complete languages
/// This might need to be in ACIR instead
#[derive(Debug, Clone)]
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}

pub fn hash_constraint_system(cs: &Circuit) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(cs.to_bytes());
    hasher.finalize_fixed().into()
}
