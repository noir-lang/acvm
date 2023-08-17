// Re-usable methods that backends can use to implement their PWG

use std::collections::HashMap;

use acir::{
    brillig::ForeignCallResult,
    circuit::{opcodes::BlockId, Opcode, OpcodeLocation},
    native_types::{Expression, Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};
use blackbox_solver::BlackBoxResolutionError;

use self::{
    arithmetic::ArithmeticSolver, brillig::BrilligSolver, directives::solve_directives,
    memory_op::MemoryOpSolver,
};
use crate::{BlackBoxFunctionSolver, Language};

use thiserror::Error;

// arithmetic
pub(crate) mod arithmetic;
// Brillig bytecode
mod brillig;
// Directives
mod directives;
// black box functions
mod blackbox;
mod memory_op;

pub use brillig::ForeignCallWaitInfo;

#[derive(Debug, Clone, PartialEq)]
pub enum ACVMStatus {
    /// All opcodes have been solved.
    Solved,

    /// The ACVM is in the process of executing the circuit.
    InProgress,

    /// The ACVM has encountered an irrecoverable error while executing the circuit and can not progress.
    /// Most commonly this will be due to an unsatisfied constraint due to invalid inputs to the circuit.
    Failure(OpcodeResolutionError),

    /// The ACVM has encountered a request for a Brillig [foreign call][acir::brillig_vm::Opcode::ForeignCall]
    /// to retrieve information from outside of the ACVM. The result of the foreign call must be passed back
    /// to the ACVM using [`ACVM::resolve_pending_foreign_call`].
    ///
    /// Once this is done, the ACVM can be restarted to solve the remaining opcodes.
    RequiresForeignCall(ForeignCallWaitInfo),
}

impl std::fmt::Display for ACVMStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ACVMStatus::Solved => write!(f, "Solved"),
            ACVMStatus::InProgress => write!(f, "In progress"),
            ACVMStatus::Failure(_) => write!(f, "Execution failure"),
            ACVMStatus::RequiresForeignCall(_) => write!(f, "Waiting on foreign call"),
        }
    }
}

// This enum represents the different cases in which an
// opcode can be unsolvable.
// The most common being that one of its input has not been
// assigned a value.
//
// TODO: ExpressionHasTooManyUnknowns is specific for arithmetic expressions
// TODO: we could have a error enum for arithmetic failure cases in that module
// TODO that can be converted into an OpcodeNotSolvable or OpcodeResolutionError enum
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum OpcodeNotSolvable {
    #[error("missing assignment for witness index {0}")]
    MissingAssignment(u32),
    #[error("expression has too many unknowns {0}")]
    ExpressionHasTooManyUnknowns(Expression),
}

/// Allows to point to a specific opcode as cause in errors.
/// Some errors don't have a specific opcode associated with them, or are created without one and added later.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ErrorLocation {
    #[default]
    Unresolved,
    Resolved(OpcodeLocation),
}

#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum OpcodeResolutionError {
    #[error("cannot solve opcode: {0}")]
    OpcodeNotSolvable(#[from] OpcodeNotSolvable),
    #[error("backend does not currently support the {0} opcode. ACVM does not currently have a fallback for this opcode.")]
    UnsupportedBlackBoxFunc(BlackBoxFunc),
    #[error("Cannot satisfy constraint {opcode_location:?}")]
    UnsatisfiedConstrain { opcode_location: ErrorLocation },
    #[error("Index out of bounds, array has size {array_size:?}, but index was {index:?}")]
    IndexOutOfBounds { opcode_location: ErrorLocation, index: u32, array_size: u32 },
    #[error("failed to solve blackbox function: {0}, reason: {1}")]
    BlackBoxFunctionFailed(BlackBoxFunc, String),
    #[error("failed to solve brillig function, reason: {0} at index {1}")]
    BrilligFunctionFailed(String, usize),
}

impl From<BlackBoxResolutionError> for OpcodeResolutionError {
    fn from(value: BlackBoxResolutionError) -> Self {
        match value {
            BlackBoxResolutionError::Failed(func, reason) => {
                OpcodeResolutionError::BlackBoxFunctionFailed(func, reason)
            }
            BlackBoxResolutionError::Unsupported(func) => {
                OpcodeResolutionError::UnsupportedBlackBoxFunc(func)
            }
        }
    }
}

pub struct ACVM<B: BlackBoxFunctionSolver> {
    status: ACVMStatus,

    backend: B,

    /// Stores the solver for memory operations acting on blocks of memory disambiguated by [block][`BlockId`].
    block_solvers: HashMap<BlockId, MemoryOpSolver>,

    /// A list of opcodes which are to be executed by the ACVM.
    opcodes: Vec<Opcode>,
    /// Index of the next opcode to be executed.
    instruction_pointer: usize,

    witness_map: WitnessMap,
}

impl<B: BlackBoxFunctionSolver> ACVM<B> {
    pub fn new(backend: B, opcodes: Vec<Opcode>, initial_witness: WitnessMap) -> Self {
        let status = if opcodes.is_empty() { ACVMStatus::Solved } else { ACVMStatus::InProgress };
        ACVM {
            status,
            backend,
            block_solvers: HashMap::default(),
            opcodes,
            instruction_pointer: 0,
            witness_map: initial_witness,
        }
    }

    /// Returns a reference to the current state of the ACVM's [`WitnessMap`].
    ///
    /// Once execution has completed, the witness map can be extracted using [`ACVM::finalize`]
    pub fn witness_map(&self) -> &WitnessMap {
        &self.witness_map
    }

    /// Returns a slice containing the opcodes of the circuit being executed.
    pub fn opcodes(&self) -> &[Opcode] {
        &self.opcodes
    }

    /// Returns the index of the current opcode to be executed.
    pub fn instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    /// Finalize the ACVM execution, returning the resulting [`WitnessMap`].
    pub fn finalize(self) -> WitnessMap {
        if self.status != ACVMStatus::Solved {
            panic!("ACVM execution is not complete: ({})", self.status);
        }
        self.witness_map
    }

    /// Updates the current status of the VM.
    /// Returns the given status.
    fn status(&mut self, status: ACVMStatus) -> ACVMStatus {
        self.status = status.clone();
        status
    }

    /// Sets the VM status to [ACVMStatus::Failure] using the provided `error`.
    /// Returns the new status.
    fn fail(&mut self, error: OpcodeResolutionError) -> ACVMStatus {
        self.status(ACVMStatus::Failure(error))
    }

    /// Sets the status of the VM to `RequiresForeignCall`.
    /// Indicating that the VM is now waiting for a foreign call to be resolved.
    fn wait_for_foreign_call(&mut self, foreign_call: ForeignCallWaitInfo) -> ACVMStatus {
        self.status(ACVMStatus::RequiresForeignCall(foreign_call))
    }

    /// Return a reference to the arguments for the next pending foreign call, if one exists.
    pub fn get_pending_foreign_call(&self) -> Option<&ForeignCallWaitInfo> {
        if let ACVMStatus::RequiresForeignCall(foreign_call) = &self.status {
            Some(foreign_call)
        } else {
            None
        }
    }

    /// Resolves a foreign call's [result][acir::brillig_vm::ForeignCallResult] using a result calculated outside of the ACVM.
    ///
    /// The ACVM can then be restarted to solve the remaining Brillig VM process as well as the remaining ACIR opcodes.
    pub fn resolve_pending_foreign_call(&mut self, foreign_call_result: ForeignCallResult) {
        if !matches!(self.status, ACVMStatus::RequiresForeignCall(_)) {
            panic!("ACVM is not expecting a foreign call response as no call was made");
        }

        // We want to inject the foreign call result into the brillig opcode which initiated the call.
        let opcode = &mut self.opcodes[self.instruction_pointer];
        let Opcode::Brillig(brillig) = opcode else {
            unreachable!("ACVM can only enter `RequiresForeignCall` state on a Brillig opcode");
        };
        brillig.foreign_call_results.push(foreign_call_result);

        // Now that the foreign call has been resolved then we can resume execution.
        self.status(ACVMStatus::InProgress);
    }

    /// Executes the ACVM's circuit until execution halts.
    ///
    /// Execution can halt due to three reasons:
    /// 1. All opcodes have been executed successfully.
    /// 2. The circuit has been found to be unsatisfiable.
    /// 2. A Brillig [foreign call][`ForeignCallWaitInfo`] has been encountered and must be resolved.
    pub fn solve(&mut self) -> ACVMStatus {
        while self.status == ACVMStatus::InProgress {
            self.solve_opcode();
        }
        self.status.clone()
    }

    pub fn solve_opcode(&mut self) -> ACVMStatus {
        let opcode = &self.opcodes[self.instruction_pointer];

        let resolution = match opcode {
            Opcode::Arithmetic(expr) => ArithmeticSolver::solve(&mut self.witness_map, expr),
            Opcode::BlackBoxFuncCall(bb_func) => {
                blackbox::solve(&self.backend, &mut self.witness_map, bb_func)
            }
            Opcode::Directive(directive) => solve_directives(&mut self.witness_map, directive),
            Opcode::MemoryInit { block_id, init } => {
                let solver = self.block_solvers.entry(*block_id).or_default();
                solver.init(init, &self.witness_map)
            }
            Opcode::MemoryOp { block_id, op } => {
                let solver = self.block_solvers.entry(*block_id).or_default();
                solver.solve_memory_op(op, &mut self.witness_map)
            }
            Opcode::Brillig(brillig) => {
                match BrilligSolver::solve(&mut self.witness_map, brillig, &self.backend) {
                    Ok(Some(foreign_call)) => return self.wait_for_foreign_call(foreign_call),
                    res => res.map(|_| ()),
                }
            }
        };
        match resolution {
            Ok(()) => {
                self.instruction_pointer += 1;
                if self.instruction_pointer == self.opcodes.len() {
                    self.status(ACVMStatus::Solved)
                } else {
                    self.status(ACVMStatus::InProgress)
                }
            }
            Err(mut error) => {
                match &mut error {
                    // If we have an index out of bounds or an unsatisfied constraint, the opcode label will be unresolved
                    // because the solvers do not have knowledge of this information.
                    // We resolve, by setting this to the corresponding opcode that we just attempted to solve.
                    OpcodeResolutionError::IndexOutOfBounds {
                        opcode_location: opcode_index,
                        ..
                    }
                    | OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: opcode_index,
                    } => {
                        *opcode_index = ErrorLocation::Resolved(OpcodeLocation::Acir(
                            self.instruction_pointer(),
                        ));
                    }
                    // If a brillig function has failed, we return an unsatisfied constraint error
                    // We intentionally ignore the brillig failure message, as there is no way to
                    // propagate this to the caller.
                    OpcodeResolutionError::BrilligFunctionFailed(
                        _,
                        brillig_instruction_pointer,
                    ) => {
                        error = OpcodeResolutionError::UnsatisfiedConstrain {
                            opcode_location: ErrorLocation::Resolved(OpcodeLocation::Brillig {
                                acir_index: self.instruction_pointer(),
                                brillig_index: *brillig_instruction_pointer,
                            }),
                        }
                    }
                    // All other errors are thrown normally.
                    _ => (),
                };
                self.fail(error)
            }
        }
    }
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
        None => Err(OpcodeResolutionError::OpcodeNotSolvable(
            OpcodeNotSolvable::MissingAssignment(any_witness_from_expression(&expr).unwrap().0),
        )),
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
        return Err(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Unresolved,
        });
    }

    Ok(())
}

// Returns one witness belonging to an expression, in no relevant order
// Returns None if the expression is const
// The function is used during partial witness generation to report unsolved witness
fn any_witness_from_expression(expr: &Expression) -> Option<Witness> {
    if expr.linear_combinations.is_empty() {
        if expr.mul_terms.is_empty() {
            None
        } else {
            Some(expr.mul_terms[0].1)
        }
    } else {
        Some(expr.linear_combinations[0].1)
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
    // attempt to transform into supported opcodes. If these are also not available
    // then a compiler error will be emitted.
    fn plonk_is_supported(_opcode: &Opcode) -> bool {
        true
    }

    match language {
        Language::R1CS => r1cs_is_supported,
        Language::PLONKCSat { .. } => plonk_is_supported,
    }
}
