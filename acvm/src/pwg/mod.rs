// Re-usable methods that backends can use to implement their PWG

use acir::{
    brillig_vm::ForeignCallResult,
    circuit::{brillig::Brillig, Opcode},
    native_types::{Expression, Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};

use self::{
    arithmetic::ArithmeticSolver, block::BlockSolver, brillig::BrilligSolver,
    directives::solve_directives,
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
mod block;

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
    RequiresForeignCall(UnresolvedBrilligCall),
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
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum OpcodeNotSolvable {
    #[error("missing assignment for witness index {0}")]
    MissingAssignment(u32),
    #[error("expression has too many unknowns {0}")]
    ExpressionHasTooManyUnknowns(Expression),
}

#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum OpcodeResolutionError {
    #[error("cannot solve opcode: {0}")]
    OpcodeNotSolvable(#[from] OpcodeNotSolvable),
    #[error("backend does not currently support the {0} opcode. ACVM does not currently have a fallback for this opcode.")]
    UnsupportedBlackBoxFunc(BlackBoxFunc),
    #[error("could not satisfy all constraints")]
    UnsatisfiedConstrain,
    #[error("failed to solve blackbox function: {0}, reason: {1}")]
    BlackBoxFunctionFailed(BlackBoxFunc, String),
    #[error("failed to solve brillig function, reason: {0}")]
    BrilligFunctionFailed(String),
}

pub struct ACVM<B: BlackBoxFunctionSolver> {
    status: ACVMStatus,

    backend: B,

    /// A list of opcodes which are to be executed by the ACVM.
    opcodes: Vec<Opcode>,
    /// Index of the next opcode to be executed.
    instruction_pointer: usize,

    witness_map: WitnessMap,
}

impl<B: BlackBoxFunctionSolver> ACVM<B> {
    pub fn new(backend: B, opcodes: Vec<Opcode>, initial_witness: WitnessMap) -> Self {
        ACVM {
            status: ACVMStatus::InProgress,
            backend,
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
            panic!("ACVM is not ready to be finalized");
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
    fn wait_for_foreign_call(
        &mut self,
        opcode: Opcode,
        foreign_call_wait_info: ForeignCallWaitInfo,
    ) -> ACVMStatus {
        let brillig = match opcode {
            Opcode::Brillig(brillig) => brillig,
            _ => unreachable!("Brillig resolution for non brillig opcode"),
        };
        let foreign_call = UnresolvedBrilligCall { brillig, foreign_call_wait_info };
        self.status(ACVMStatus::RequiresForeignCall(foreign_call))
    }

    /// Return a reference to the arguments for the next pending foreign call, if one exists.
    pub fn get_pending_foreign_call(&self) -> Option<&ForeignCallWaitInfo> {
        if let ACVMStatus::RequiresForeignCall(foreign_call) = &self.status {
            Some(&foreign_call.foreign_call_wait_info)
        } else {
            None
        }
    }

    /// Resolves a pending foreign call using a result calculated outside of the ACVM.
    pub fn resolve_pending_foreign_call(&mut self, foreign_call_result: ForeignCallResult) {
        let foreign_call = if let ACVMStatus::RequiresForeignCall(foreign_call) = &self.status {
            // TODO: We can avoid this clone
            foreign_call.clone()
        } else {
            panic!("no foreign call")
        };
        let resolved_brillig = foreign_call.resolve(foreign_call_result);

        // Overwrite the brillig opcode with a new one with the foreign call response.
        self.opcodes[self.instruction_pointer] = Opcode::Brillig(resolved_brillig);

        // Now that the foreign call has been resolved then we can resume execution.
        self.status(ACVMStatus::InProgress);
    }

    /// Executes the ACVM's circuit until execution halts.
    ///
    /// Execution can halt due to three reasons:
    /// 1. All opcodes have been executed successfully.
    /// 2. The circuit has been found to be unsatisfiable.
    /// 2. A Brillig [foreign call][`UnresolvedBrilligCall`] has been encountered and must be resolved.
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
            Opcode::Block(block) | Opcode::ROM(block) | Opcode::RAM(block) => {
                BlockSolver.solve(&mut self.witness_map, &block.trace)
            }
            Opcode::Brillig(brillig) => {
                let resolution = BrilligSolver::solve(&mut self.witness_map, brillig);

                match resolution {
                    Ok(OpcodeResolution::InProgressBrillig(foreign_call_wait_info)) => {
                        return self.wait_for_foreign_call(opcode.clone(), foreign_call_wait_info)
                    }
                    res => res,
                }
            }
        };
        match resolution {
            Ok(OpcodeResolution::Solved) => {
                self.instruction_pointer += 1;
                if self.instruction_pointer == self.opcodes.len() {
                    self.status(ACVMStatus::Solved)
                } else {
                    self.status(ACVMStatus::InProgress)
                }
            }
            Ok(OpcodeResolution::InProgress) => {
                unreachable!("Opcodes should be immediately solvable");
            }
            Ok(OpcodeResolution::InProgressBrillig(_)) => {
                unreachable!("Handled above")
            }
            Ok(OpcodeResolution::Stalled(not_solvable)) => {
                self.fail(OpcodeResolutionError::OpcodeNotSolvable(not_solvable))
            }
            Err(error) => self.fail(error),
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
