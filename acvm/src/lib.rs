// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub mod compiler;
pub mod pwg;
use std::collections::BTreeMap;

use acir::{
    circuit::{directives::Directive, opcodes::BlackBoxFuncCall, Circuit, Opcode},
    native_types::{Expression, Witness},
    BlackBoxFunc,
};
use blake2::digest::FixedOutput;

use crate::pwg::{arithmetic::ArithmeticSolver, logic::LogicSolver};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use thiserror::Error;

// re-export acir
pub use acir;
pub use acir::FieldElement;

#[derive(PartialEq, Eq, Debug)]
pub enum OpcodeResolution {
    Resolved, // Opcode is solved
    Unsolved, // Opcode cannot be solved
}

#[derive(PartialEq, Eq, Debug, Error)]
pub enum OpcodeResolutionError {
    #[error("{0}")]
    UnknownError(String),
    #[error("backend does not currently support the {0} opcode. ACVM does not currently fall back to arithmetic gates.")]
    UnsupportedBlackBoxFunc(BlackBoxFunc),
    #[error("could not satisfy all constraints")]
    UnsatisfiedConstrain,
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
        let mut unsolved_gates: Vec<Opcode> = Vec::new();

        for gate in gates.into_iter() {
            let resolution = match &gate {
                Opcode::Arithmetic(expr) => ArithmeticSolver::solve(initial_witness, expr),
                Opcode::BlackBoxFuncCall(bb_func) => {
                    Self::solve_blackbox_function_call(initial_witness, bb_func)
                }
                Opcode::Directive(directive) => Self::solve_directives(initial_witness, directive),
            }?;

            if resolution == OpcodeResolution::Unsolved {
                unsolved_gates.push(gate);
            }
        }
        self.solve(initial_witness, unsolved_gates)
    }

    fn solve_blackbox_function_call(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<OpcodeResolution, OpcodeResolutionError>;

    fn solve_range_opcode(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        // TODO: this consistency check can be moved to a general function
        let defined_input_size = BlackBoxFunc::RANGE
            .definition()
            .input_size
            .fixed_size()
            .expect("infallible: input for range gate is fixed");

        if func_call.inputs.len() != defined_input_size as usize {
            return Err(OpcodeResolutionError::UnknownError(
                "defined input size does not equal given input size".to_string(),
            ));
        }

        // For the range constraint, we know that the input size should be one
        assert_eq!(defined_input_size, 1);

        let input = func_call
            .inputs
            .first()
            .expect("infallible: checked that input size is 1");

        let w_value = match initial_witness.get(&input.witness) {
            Some(value) => value,
            None => return Ok(OpcodeResolution::Unsolved),
        };

        if w_value.num_bits() > input.num_bits {
            return Err(OpcodeResolutionError::UnsatisfiedConstrain);
        }

        Ok(OpcodeResolution::Resolved)
    }

    fn solve_logic_opcode(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        match func_call.name {
            BlackBoxFunc::AND => LogicSolver::solve_and_gate(initial_witness, &func_call),
            BlackBoxFunc::XOR => LogicSolver::solve_xor_gate(initial_witness, &func_call),
            _ => Err(OpcodeResolutionError::UnknownError(format!(
                "expected a logic opcode, but instead got {:?}",
                func_call.name,
            ))),
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
    ) -> Option<FieldElement> {
        let mut result = expr.q_c;

        for term in &expr.linear_combinations {
            let coefficient = term.0;
            let variable = term.1;

            // Get the value assigned to that variable
            let assignment = match initial_witness.get(&variable) {
                Some(value) => *value,
                None => return None,
            };

            result += coefficient * assignment;
        }

        for term in &expr.mul_terms {
            let coefficient = term.0;
            let lhs_variable = term.1;
            let rhs_variable = term.2;

            // Get the values assigned to those variables
            let (lhs_assignment, rhs_assignment) = match (
                initial_witness.get(&lhs_variable),
                initial_witness.get(&rhs_variable),
            ) {
                (Some(lhs), Some(rhs)) => (*lhs, *rhs),
                (_, _) => return None,
            };

            result += coefficient * lhs_assignment * rhs_assignment;
        }

        Some(result)
    }

    fn solve_directives(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        directive: &Directive,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        match directive {
            Directive::Invert { x, result } => {
                let val = match initial_witness.get(x) {
                    Some(value) => value,
                    None => return Ok(OpcodeResolution::Unsolved),
                };

                let inverse = val.inverse();
                initial_witness.insert(*result, inverse);
                return Ok(OpcodeResolution::Resolved);
            }
            Directive::Quotient {
                a,
                b,
                q,
                r,
                predicate,
            } => {
                let (val_a, val_b) = match (
                    Self::get_value(a, initial_witness),
                    Self::get_value(b, initial_witness),
                ) {
                    (Some(a), Some(b)) => (a, b),
                    (_, _) => return Ok(OpcodeResolution::Unsolved),
                };

                let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                let int_b = BigUint::from_bytes_be(&val_b.to_bytes());

                // If the predicate is `None`, then we simply return the value 1
                // If the predicate is `Some` but we cannot find a value, then we return unresolved
                let pred_value = match predicate {
                    Some(pred) => match Self::get_value(pred, initial_witness) {
                        Some(val) => val,
                        None => return Ok(OpcodeResolution::Unsolved),
                    },
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

                Ok(OpcodeResolution::Resolved)
            }
            Directive::Truncate { a, b, c, bit_size } => {
                let val_a = match initial_witness.get(a) {
                    Some(value) => value,
                    None => return Ok(OpcodeResolution::Unsolved),
                };

                let pow: BigUint = BigUint::one() << bit_size;

                let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                let int_b: BigUint = &int_a % &pow;
                let int_c: BigUint = (&int_a - &int_b) / &pow;

                initial_witness
                    .insert(*b, FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()));
                initial_witness
                    .insert(*c, FieldElement::from_be_bytes_reduce(&int_c.to_bytes_be()));

                Ok(OpcodeResolution::Resolved)
            }
            Directive::ToBits { a, b, bit_size } => {
                let val_a = match Self::get_value(a, initial_witness) {
                    Some(value) => value,
                    None => return Ok(OpcodeResolution::Unsolved),
                };

                let a_big = BigUint::from_bytes_be(&val_a.to_bytes());
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

                Ok(OpcodeResolution::Resolved)
            }
            Directive::ToBytes { a, b, byte_size } => {
                let val_a = match Self::get_value(a, initial_witness) {
                    Some(value) => value,
                    None => return Ok(OpcodeResolution::Unsolved),
                };

                let mut a_bytes = val_a.to_bytes();
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

                Ok(OpcodeResolution::Resolved)
            }
            Directive::Oddrange { a, b, r, bit_size } => {
                let val_a = match initial_witness.get(a) {
                    Some(value) => value,
                    None => return Ok(OpcodeResolution::Unsolved),
                };

                let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
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

                Ok(OpcodeResolution::Resolved)
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
