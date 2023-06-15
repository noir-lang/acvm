// Re-usable methods that backends can use to implement their PWG

use crate::{BlackBoxFunctionSolver, Language};
use acir::{
    brillig_vm::ForeignCallResult,
    circuit::brillig::Brillig,
    circuit::opcodes::{Opcode, OracleData},
    native_types::{Expression, Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};

use self::{
    arithmetic::ArithmeticSolver, brillig::BrilligSolver, directives::solve_directives,
    oracle::OracleSolver,
};

use thiserror::Error;

// arithmetic
pub(crate) mod arithmetic;
// Brillig bytecode
mod brillig;
// Directives
mod directives;
// black box functions
mod blackbox;
mod block;
mod oracle;

// Re-export `Blocks` so that it can be passed to `pwg::solve`
pub use block::Blocks;
pub use brillig::ForeignCallWaitInfo;

#[derive(Debug, PartialEq)]
pub enum PartialWitnessGeneratorStatus {
    /// All opcodes have been solved.
    Solved,

    /// The `PartialWitnessGenerator` has encountered a request for [oracle data][Opcode::Oracle] or a Brillig [foreign call][acir::brillig_vm::Opcode::ForeignCall].
    ///
    /// Both of these opcodes require information from outside of the ACVM to be inserted before restarting execution.
    /// [`Opcode::Oracle`] and [`Opcode::Brillig`] opcodes require the return values to be inserted slightly differently.
    /// `Oracle` opcodes expect their return values to be written directly into the witness map whereas a `Brillig` foreign call
    /// result is inserted into the `Brillig` opcode which made the call using [`UnresolvedBrilligCall::resolve`].
    /// (Note: this means that the updated opcode must then be passed back into the ACVM to be processed further.)
    ///
    /// Once this is done, the `PartialWitnessGenerator` can be restarted to solve the remaining opcodes.
    RequiresOracleData {
        required_oracle_data: Vec<OracleData>,
        unsolved_opcodes: Vec<Opcode>,
        unresolved_brillig_calls: Vec<UnresolvedBrilligCall>,
    },
}

#[derive(Debug, PartialEq)]
pub enum OpcodeResolution {
    /// The opcode is resolved
    Solved,
    /// The opcode is not solvable
    Stalled(OpcodeNotSolvable),
    /// The opcode is not solvable but could resolved some witness
    InProgress,
    /// The brillig oracle opcode is not solved but could be resolved given some values
    InProgressBrillig(brillig::ForeignCallWaitInfo),
}

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
    OpcodeNotSolvable(#[from] OpcodeNotSolvable),
    #[error("backend does not currently support the {0} opcode. ACVM does not currently have a fallback for this opcode.")]
    UnsupportedBlackBoxFunc(BlackBoxFunc),
    #[error("could not satisfy all constraints")]
    UnsatisfiedConstrain,
    #[error("expected {0} inputs for function {1}, but got {2}")]
    IncorrectNumFunctionArguments(usize, BlackBoxFunc, usize),
    #[error("failed to solve blackbox function: {0}, reason: {1}")]
    BlackBoxFunctionFailed(BlackBoxFunc, String),
    #[error("failed to solve brillig function, reason: {0}")]
    BrilligFunctionFailed(String),
}

/// Executes a [`Circuit`] against an [initial witness][`WitnessMap`] to calculate the solved partial witness.
pub fn solve(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    blocks: &mut Blocks,
    mut opcode_to_solve: Vec<Opcode>,
) -> Result<PartialWitnessGeneratorStatus, OpcodeResolutionError> {
    let mut unresolved_opcodes: Vec<Opcode> = Vec::new();
    let mut unresolved_oracles: Vec<OracleData> = Vec::new();
    let mut unresolved_brillig_calls: Vec<UnresolvedBrilligCall> = Vec::new();
    while !opcode_to_solve.is_empty() || !unresolved_oracles.is_empty() {
        unresolved_opcodes.clear();
        let mut stalled = true;
        let mut opcode_not_solvable = None;
        for opcode in &opcode_to_solve {
            let mut solved_oracle_data = None;
            let resolution = match opcode {
                Opcode::Arithmetic(expr) => ArithmeticSolver::solve(initial_witness, expr),
                Opcode::BlackBoxFuncCall(bb_func) => {
                    blackbox::solve(backend, initial_witness, bb_func)
                }
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
                Opcode::Brillig(brillig) => BrilligSolver::solve(initial_witness, brillig),
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
                Ok(OpcodeResolution::InProgressBrillig(oracle_wait_info)) => {
                    stalled = false;
                    // InProgressBrillig Oracles must be externally re-solved
                    let brillig = match opcode {
                        Opcode::Brillig(brillig) => brillig.clone(),
                        _ => unreachable!("Brillig resolution for non brillig opcode"),
                    };
                    unresolved_brillig_calls.push(UnresolvedBrilligCall {
                        brillig,
                        foreign_call_wait_info: oracle_wait_info,
                    })
                }
                Ok(OpcodeResolution::Stalled(not_solvable)) => {
                    if opcode_not_solvable.is_none() {
                        // we keep track of the first unsolvable opcode
                        opcode_not_solvable = Some(not_solvable);
                    }
                    // We push those opcodes not solvable to the back as
                    // it could be because the opcodes are out of order, i.e. this assignment
                    // relies on a later opcodes' results
                    if let Some(oracle_data) = solved_oracle_data {
                        unresolved_opcodes.push(Opcode::Oracle(oracle_data));
                    } else {
                        unresolved_opcodes.push(opcode.clone());
                    }
                }
                Err(OpcodeResolutionError::OpcodeNotSolvable(_)) => {
                    unreachable!("ICE - Result should have been converted to GateResolution")
                }
                Err(err) => return Err(err),
            }
        }
        // We have oracles that must be externally resolved
        if !unresolved_oracles.is_empty() || !unresolved_brillig_calls.is_empty() {
            return Ok(PartialWitnessGeneratorStatus::RequiresOracleData {
                required_oracle_data: unresolved_oracles,
                unsolved_opcodes: unresolved_opcodes,
                unresolved_brillig_calls,
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
    initial_witness: &WitnessMap,
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
    initial_witness: &WitnessMap,
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

/// Inserts `value` into the initial witness map under the index `witness`.
///
/// Returns an error if there was already a value in the map
/// which does not match the value that one is about to insert
pub fn insert_value(
    witness: &Witness,
    value_to_insert: FieldElement,
    initial_witness: &mut WitnessMap,
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

/// A Brillig VM process has requested the caller to solve a [foreign call][brillig_vm::Opcode::ForeignCall] externally
/// and to re-run the process with the foreign call's resolved outputs.
#[derive(Debug, PartialEq, Clone)]
pub struct UnresolvedBrilligCall {
    /// The current Brillig VM process that has been paused.
    /// This process will be updated by the caller after resolving a foreign call's result.
    ///
    /// This can be done using [`UnresolvedBrilligCall::resolve`].
    pub brillig: Brillig,
    /// Inputs for a pending foreign call required to restart bytecode processing.
    pub foreign_call_wait_info: brillig::ForeignCallWaitInfo,
}

impl UnresolvedBrilligCall {
    /// Inserts the [foreign call's result][acir::brillig_vm::ForeignCallResult] into the calling [`Brillig` opcode][Brillig].
    ///
    /// The [ACVM][solve] can then be restarted with the updated [Brillig opcode][Opcode::Brillig]
    /// to solve the remaining Brillig VM process as well as the remaining ACIR opcodes.
    pub fn resolve(mut self, foreign_call_result: ForeignCallResult) -> Brillig {
        self.brillig.foreign_call_results.push(foreign_call_result);
        self.brillig
    }
}

#[deprecated(
    note = "For backwards compatibility, this method allows you to derive _sensible_ defaults for opcode support based on the np language. \n Backends should simply specify what they support."
)]
// This is set to match the previous functionality that we had
// Where we could deduce what opcodes were supported
// by knowing the np complete language
pub fn default_is_opcode_supported(language: Language) -> fn(&Opcode) -> bool {
    // R1CS does not support any of the opcode except Arithmetic by default.
    // The compiler will replace those that it can -- ie range, xor, and
    fn r1cs_is_supported(opcode: &Opcode) -> bool {
        matches!(opcode, Opcode::Arithmetic(_))
    }

    // PLONK supports most of the opcodes by default
    // The ones which are not supported, the acvm compiler will
    // attempt to transform into supported gates. If these are also not available
    // then a compiler error will be emitted.
    fn plonk_is_supported(opcode: &Opcode) -> bool {
        !matches!(opcode, Opcode::Block(_))
    }

    match language {
        Language::R1CS => r1cs_is_supported,
        Language::PLONKCSat { .. } => plonk_is_supported,
    }
}
