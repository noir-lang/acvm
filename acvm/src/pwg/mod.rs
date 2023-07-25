// Re-usable methods that backends can use to implement their PWG

use std::collections::HashMap;

use acir::{
    brillig::ForeignCallResult,
    circuit::{brillig::Brillig, opcodes::BlockId, Opcode, OpcodeLabel},
    native_types::{Expression, Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};
use blackbox_solver::BlackBoxResolutionError;

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
    RequiresForeignCall,
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
    UnsatisfiedConstrain { opcode_label: OpcodeLabel },
    #[error("failed to solve blackbox function: {0}, reason: {1}")]
    BlackBoxFunctionFailed(BlackBoxFunc, String),
    #[error("failed to solve brillig function, reason: {0}")]
    BrilligFunctionFailed(String),
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

    /// Stores the solver for each [block][`Opcode::Block`] opcode. This persists their internal state to prevent recomputation.
    block_solvers: HashMap<BlockId, BlockSolver>,

    /// A list of opcodes which are to be executed by the ACVM, along with their label
    ///
    /// Note that this doesn't include any opcodes which are waiting on a pending foreign call.
    opcodes_and_labels: Vec<(Opcode, OpcodeLabel)>,

    witness_map: WitnessMap,

    /// A list of foreign calls which must be resolved before the ACVM can resume execution.
    pending_foreign_calls: Vec<UnresolvedBrilligCall>,

    /// Map from a canonical hash of an unresolved Brillig call to its opcode label.
    pending_brillig_label_maps: HashMap<UnresolvedBrilligCallHash, OpcodeLabel>,
}

impl<B: BlackBoxFunctionSolver> ACVM<B> {
    pub fn new(backend: B, opcodes: Vec<Opcode>, initial_witness: WitnessMap) -> Self {
        let opcodes_and_labels = opcodes
            .iter()
            .enumerate()
            .map(|(opcode_index, opcode)| {
                (opcode.clone(), OpcodeLabel::Resolved(opcode_index as u64))
            })
            .collect();
        ACVM {
            status: ACVMStatus::InProgress,
            backend,
            block_solvers: HashMap::default(),
            opcodes_and_labels,
            witness_map: initial_witness,
            pending_foreign_calls: Vec::new(),
            pending_brillig_label_maps: HashMap::new(),
        }
    }

    /// Returns a reference to the current state of the ACVM's [`WitnessMap`].
    ///
    /// Once execution has completed, the witness map can be extracted using [`ACVM::finalize`]
    pub fn witness_map(&self) -> &WitnessMap {
        &self.witness_map
    }

    /// Returns a slice containing the opcodes which remain to be solved.
    ///
    /// Note: this doesn't include any opcodes which are waiting on a pending foreign call.
    pub fn unresolved_opcodes(&self) -> &[(Opcode, OpcodeLabel)] {
        &self.opcodes_and_labels
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

    /// Finalize the ACVM execution, returning the resulting [`WitnessMap`].
    pub fn finalize(self) -> WitnessMap {
        if self.status != ACVMStatus::Solved {
            panic!("ACVM is not ready to be finalized");
        }
        self.witness_map
    }

    /// Return a reference to the arguments for the next pending foreign call, if one exists.
    pub fn get_pending_foreign_call(&self) -> Option<&ForeignCallWaitInfo> {
        self.pending_foreign_calls.first().map(|foreign_call| &foreign_call.foreign_call_wait_info)
    }

    /// Resolves a pending foreign call using a result calculated outside of the ACVM.
    pub fn resolve_pending_foreign_call(&mut self, foreign_call_result: ForeignCallResult) {
        // Remove the first foreign call and inject the result to create a new opcode.
        let foreign_call = self.pending_foreign_calls.remove(0);
        let resolved_brillig = foreign_call.resolve(foreign_call_result);

        // Mark this opcode to be executed next.
        let hash = canonical_brillig_hash(&resolved_brillig);
        self.opcodes_and_labels
            .insert(0, (Opcode::Brillig(resolved_brillig), self.pending_brillig_label_maps[&hash]));
    }

    /// Executes the ACVM's circuit until execution halts.
    ///
    /// Execution can halt due to three reasons:
    /// 1. All opcodes have been executed successfully.
    /// 2. The circuit has been found to be unsatisfiable.
    /// 2. A Brillig [foreign call][`UnresolvedBrilligCall`] has been encountered and must be resolved.
    pub fn solve(&mut self) -> ACVMStatus {
        // TODO: Prevent execution with outstanding foreign calls?
        let mut unresolved_opcodes: Vec<(Opcode, OpcodeLabel)> = Vec::new();
        while !self.opcodes_and_labels.is_empty() {
            unresolved_opcodes.clear();
            let mut stalled = true;
            let mut opcode_not_solvable = None;
            for (opcode, opcode_label) in &self.opcodes_and_labels {
                let mut resolution = match opcode {
                    Opcode::Arithmetic(expr) => {
                        ArithmeticSolver::solve(&mut self.witness_map, expr)
                    }
                    Opcode::BlackBoxFuncCall(bb_func) => {
                        blackbox::solve(&self.backend, &mut self.witness_map, bb_func)
                    }
                    Opcode::Directive(directive) => {
                        solve_directives(&mut self.witness_map, directive)
                    }
                    Opcode::Block(block) | Opcode::ROM(block) | Opcode::RAM(block) => {
                        let solver = self.block_solvers.entry(block.id).or_default();
                        solver.solve(&mut self.witness_map, &block.trace)
                    }
                    Opcode::Brillig(brillig) => {
                        BrilligSolver::solve(&mut self.witness_map, brillig, &self.backend)
                    }
                    Opcode::MemoryInit { block_id, init } => {
                        let solver = self.block_solvers.entry(*block_id).or_default();
                        solver.init(init, &self.witness_map);
                        Ok(OpcodeResolution::Solved)
                    }
                    Opcode::MemoryOp { block_id, op } => {
                        let solver = self.block_solvers.entry(*block_id).or_default();
                        let result = solver.solve_memory_op(op, &mut self.witness_map);
                        match result {
                            Ok(_) => Ok(OpcodeResolution::Solved),
                            Err(err) => Err(err),
                        }
                    }
                };

                // If we have an unsatisfied constraint, the opcode label will be unresolved
                // because the solvers do not have knowledge of this information.
                // We resolve, by setting this to the corresponding opcode that we just attempted to solve.
                if let Err(OpcodeResolutionError::UnsatisfiedConstrain {
                    opcode_label: opcode_index,
                }) = &mut resolution
                {
                    *opcode_index = *opcode_label;
                }
                // If a brillig function has failed, we return an unsatisfied constraint error
                // We intentionally ignore the brillig failure message, as there is no way to
                // propagate this to the caller.
                if let Err(OpcodeResolutionError::BrilligFunctionFailed(_)) = &mut resolution {
                    //
                    // std::mem::swap(x, y)
                    resolution = Err(OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_label: *opcode_label,
                    })
                }

                match resolution {
                    Ok(OpcodeResolution::Solved) => {
                        stalled = false;
                    }
                    Ok(OpcodeResolution::InProgress) => {
                        stalled = false;
                        unresolved_opcodes.push((opcode.clone(), *opcode_label));
                    }
                    Ok(OpcodeResolution::InProgressBrillig(oracle_wait_info)) => {
                        stalled = false;
                        // InProgressBrillig Oracles must be externally re-solved
                        let brillig = match &opcode {
                            Opcode::Brillig(brillig) => brillig.clone(),
                            _ => unreachable!("Brillig resolution for non brillig opcode"),
                        };
                        let hash = canonical_brillig_hash(&brillig);
                        self.pending_brillig_label_maps.insert(hash, *opcode_label);
                        self.pending_foreign_calls.push(UnresolvedBrilligCall {
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
                        unresolved_opcodes.push((opcode.clone(), *opcode_label));
                    }
                    Err(OpcodeResolutionError::OpcodeNotSolvable(_)) => {
                        unreachable!("ICE - Result should have been converted to GateResolution")
                    }
                    Err(error) => return self.fail(error),
                }
            }

            // Before potentially ending execution, we must save the list of opcodes which remain to be solved.
            std::mem::swap(&mut self.opcodes_and_labels, &mut unresolved_opcodes);

            // We have oracles that must be externally resolved
            if self.get_pending_foreign_call().is_some() {
                return self.status(ACVMStatus::RequiresForeignCall);
            }

            // We are stalled because of an opcode being bad
            if stalled && !self.opcodes_and_labels.is_empty() {
                let error = OpcodeResolutionError::OpcodeNotSolvable(
                    opcode_not_solvable
                        .expect("infallible: cannot be stalled and None at the same time"),
                );
                return self.fail(error);
            }
        }
        self.status(ACVMStatus::Solved)
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
            opcode_label: OpcodeLabel::Unresolved,
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

/// Canonically hashes the Brillig struct.
///
/// Some Brillig instances may or may not be resolved, so we do
/// not hash the `foreign_call_results`.
///
/// This gives us a consistent hash that will be used to track `Brillig`
/// even when it is put back into the list of opcodes out of order.
/// This happens when we resolve a Brillig opcode call.
pub fn canonical_brillig_hash(brillig: &Brillig) -> UnresolvedBrilligCallHash {
    let mut serialized_vector = rmp_serde::to_vec(&brillig.inputs).unwrap();
    serialized_vector.extend(rmp_serde::to_vec(&brillig.outputs).unwrap());
    serialized_vector.extend(rmp_serde::to_vec(&brillig.bytecode).unwrap());
    serialized_vector.extend(rmp_serde::to_vec(&brillig.predicate).unwrap());

    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let mut hasher = DefaultHasher::new();
    hasher.write(&serialized_vector);
    hasher.finish()
}

/// Hash of an unresolved brillig call instance
pub(crate) type UnresolvedBrilligCallHash = u64;
