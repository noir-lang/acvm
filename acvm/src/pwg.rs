// Re-usable methods that backends can use to implement their PWG

use crate::{OpcodeNotSolvable, OpcodeResolutionError, PartialWitnessGenerator};
use acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput, Opcode, OracleData},
    native_types::{Expression, Witness},
    BlackBoxFunc, FieldElement,
};
use std::collections::BTreeMap;

use self::{
    arithmetic::ArithmeticSolver, block::Blocks, directives::solve_directives, oracle::OracleSolver,
};

// arithmetic
pub mod arithmetic;
// Directives
pub mod directives;
// black box functions
pub mod block;
pub mod hash;
pub mod logic;
pub mod oracle;
pub mod range;
pub mod signature;
pub mod sorting;

#[derive(Debug, PartialEq)]
pub enum PartialWitnessGeneratorStatus {
    /// All opcodes have been solved.
    Solved,

    /// The `PartialWitnessGenerator` has encountered a request for [oracle data][Opcode::Oracle].
    ///
    /// The caller must resolve these opcodes externally and insert the results into the intermediate witness.
    /// Once this is done, the `PartialWitnessGenerator` can be restarted to solve the remaining opcodes.
    RequiresOracleData { required_oracle_data: Vec<OracleData>, unsolved_opcodes: Vec<Opcode> },
}

#[derive(Debug, PartialEq)]
pub enum OpcodeResolution {
    /// The opcode is resolved
    Solved,
    /// The opcode is not solvable
    Stalled(OpcodeNotSolvable),
    /// The opcode is not solvable but could resolved some witness
    InProgress,
}

/// Check if all of the inputs to the function have assignments
///
/// Returns the first missing assignment if any are missing
fn first_missing_assignment(
    witness_assignments: &BTreeMap<Witness, FieldElement>,
    inputs: &Vec<FunctionInput>,
) -> Option<Witness> {
    inputs.iter().find_map(|input| {
        if witness_assignments.contains_key(&input.witness) {
            None
        } else {
            Some(input.witness)
        }
    })
}

fn is_stalled(
    witness_assignments: &BTreeMap<Witness, FieldElement>,
    inputs: &Vec<FunctionInput>,
) -> bool {
    inputs.iter().all(|input| witness_assignments.contains_key(&input.witness))
}

pub fn solve(
    backend: impl PartialWitnessGenerator,
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    blocks: &mut Blocks,
    mut opcode_to_solve: Vec<Opcode>,
) -> Result<PartialWitnessGeneratorStatus, OpcodeResolutionError> {
    let mut unresolved_opcodes: Vec<Opcode> = Vec::new();
    let mut unresolved_oracles: Vec<OracleData> = Vec::new();
    while !opcode_to_solve.is_empty() || !unresolved_oracles.is_empty() {
        unresolved_opcodes.clear();
        let mut stalled = true;
        let mut opcode_not_solvable = None;
        for opcode in &opcode_to_solve {
            let mut solved_oracle_data = None;
            let resolution = match opcode {
                Opcode::Arithmetic(expr) => ArithmeticSolver::solve(initial_witness, expr),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall { inputs, .. })
                    if !is_stalled(initial_witness, inputs) =>
                {
                    if let Some(unassigned_witness) =
                        first_missing_assignment(initial_witness, inputs)
                    {
                        Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::MissingAssignment(
                            unassigned_witness.0,
                        )))
                    } else {
                        // This only exists because Rust won't let us bind in an pattern guard.
                        // See https://github.com/rust-lang/rust/issues/51114
                        unreachable!("Only reachable if the blackbox is stalled")
                    }
                }
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::AES,
                    inputs,
                    outputs,
                }) => backend.aes(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::AND,
                    inputs,
                    outputs,
                }) => backend.and(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::XOR,
                    inputs,
                    outputs,
                }) => backend.xor(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::RANGE,
                    inputs,
                    outputs,
                }) => backend.range(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::SHA256,
                    inputs,
                    outputs,
                }) => backend.sha256(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::Blake2s,
                    inputs,
                    outputs,
                }) => backend.blake2s(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::ComputeMerkleRoot,
                    inputs,
                    outputs,
                }) => backend.compute_merkle_root(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::SchnorrVerify,
                    inputs,
                    outputs,
                }) => backend.schnorr_verify(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::Pedersen,
                    inputs,
                    outputs,
                }) => backend.pedersen(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::HashToField128Security,
                    inputs,
                    outputs,
                }) => backend.hash_to_field128_security(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::EcdsaSecp256k1,
                    inputs,
                    outputs,
                }) => backend.ecdsa_secp256k1(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::FixedBaseScalarMul,
                    inputs,
                    outputs,
                }) => backend.fixed_base_scalar_mul(initial_witness, inputs, outputs),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                    name: BlackBoxFunc::Keccak256,
                    inputs,
                    outputs,
                }) => backend.keccak256(initial_witness, inputs, outputs),
                Opcode::Directive(directive) => solve_directives(initial_witness, directive),
                Opcode::Block(block) | Opcode::ROM(block) | Opcode::RAM(block) => {
                    blocks.solve(block.id, &block.trace, initial_witness)
                }
                Opcode::Oracle(data) => {
                    let mut data_clone = data.clone();
                    let result = OracleSolver::solve(initial_witness, &mut data_clone)?;
                    solved_oracle_data = Some(data_clone);
                    Ok(result)
                }
            };
            match resolution {
                Ok(OpcodeResolution::Solved) => {
                    stalled = false;
                }
                Ok(OpcodeResolution::InProgress) => {
                    stalled = false;
                    // InProgress Oracles must be externally re-solved
                    if let Some(oracle) = solved_oracle_data {
                        unresolved_oracles.push(oracle);
                    } else {
                        unresolved_opcodes.push(opcode.clone());
                    }
                }
                Ok(OpcodeResolution::Stalled(not_solvable)) => {
                    if opcode_not_solvable.is_none() {
                        // we keep track of the first unsolvable opcode
                        opcode_not_solvable = Some(not_solvable);
                    }
                    // We push those opcodes not solvable to the back as
                    // it could be because the opcodes are out of order, i.e. this assignment
                    // relies on a later opcodes' results
                    unresolved_opcodes.push(match solved_oracle_data {
                        Some(oracle_data) => Opcode::Oracle(oracle_data),
                        None => opcode.clone(),
                    });
                }
                Err(OpcodeResolutionError::OpcodeNotSolvable(_)) => {
                    unreachable!("ICE - Result should have been converted to GateResolution")
                }
                Err(err) => return Err(err),
            }
        }
        // We have oracles that must be externally resolved
        if !unresolved_oracles.is_empty() {
            return Ok(PartialWitnessGeneratorStatus::RequiresOracleData {
                required_oracle_data: unresolved_oracles,
                unsolved_opcodes: unresolved_opcodes,
            });
        }
        // We are stalled because of an opcode being bad
        if stalled && !unresolved_opcodes.is_empty() {
            return Err(OpcodeResolutionError::OpcodeNotSolvable(
                opcode_not_solvable
                    .expect("infallible: cannot be stalled and None at the same time"),
            ));
        }
        std::mem::swap(&mut opcode_to_solve, &mut unresolved_opcodes);
    }
    Ok(PartialWitnessGeneratorStatus::Solved)
}

// Returns the concrete value for a particular witness
// If the witness has no assignment, then
// an error is returned
pub fn witness_to_value(
    initial_witness: &BTreeMap<Witness, FieldElement>,
    witness: Witness,
) -> Result<&FieldElement, OpcodeResolutionError> {
    match initial_witness.get(&witness) {
        Some(value) => Ok(value),
        None => Err(OpcodeNotSolvable::MissingAssignment(witness.0).into()),
    }
}

// TODO: There is an issue open to decide on whether we need to get values from Expressions
// TODO versus just getting values from Witness
pub fn get_value(
    expr: &Expression,
    initial_witness: &BTreeMap<Witness, FieldElement>,
) -> Result<FieldElement, OpcodeResolutionError> {
    let expr = ArithmeticSolver::evaluate(expr, initial_witness);
    match expr.to_const() {
        Some(value) => Ok(value),
        None => {
            Err(OpcodeResolutionError::OpcodeNotSolvable(OpcodeNotSolvable::MissingAssignment(
                ArithmeticSolver::any_witness_from_expression(&expr).unwrap().0,
            )))
        }
    }
}

// Inserts `value` into the initial witness map
// under the key of `witness`.
// Returns an error, if there was already a value in the map
// which does not match the value that one is about to insert
fn insert_value(
    witness: &Witness,
    value_to_insert: FieldElement,
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
) -> Result<(), OpcodeResolutionError> {
    let optional_old_value = initial_witness.insert(*witness, value_to_insert);

    let old_value = match optional_old_value {
        Some(old_value) => old_value,
        None => return Ok(()),
    };

    if old_value != value_to_insert {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain);
    }

    Ok(())
}
