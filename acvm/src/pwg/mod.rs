// Re-usable methods that backends can use to implement their PWG

use crate::{Language, PartialWitnessGenerator};
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
    backend: &impl PartialWitnessGenerator,
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
            let mut solved_brillig_data = None;
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
                Opcode::Brillig(brillig) => {
                    let mut brillig_clone = brillig.clone();
                    let result = BrilligSolver::solve(initial_witness, &mut brillig_clone);
                    solved_brillig_data = Some(brillig_clone);
                    result
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
                    } else if let Some(brillig) = solved_brillig_data {
                        unresolved_opcodes.push(Opcode::Brillig(brillig));
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acir::{
        brillig_vm::{self, BinaryFieldOp, RegisterIndex, RegisterOrMemory, Value},
        circuit::{
            brillig::{Brillig, BrilligInputs, BrilligOutputs},
            directives::Directive,
            opcodes::{FunctionInput, OracleData},
            Opcode,
        },
        native_types::{Expression, Witness, WitnessMap},
        FieldElement,
    };

    use crate::{
        pwg::{self, block::Blocks, OpcodeResolution, PartialWitnessGeneratorStatus},
        OpcodeResolutionError, PartialWitnessGenerator,
    };

    struct StubbedPwg;

    impl PartialWitnessGenerator for StubbedPwg {
        fn schnorr_verify(
            &self,
            _initial_witness: &mut WitnessMap,
            _public_key_x: &FunctionInput,
            _public_key_y: &FunctionInput,
            _signature: &[FunctionInput],
            _message: &[FunctionInput],
            _output: &Witness,
        ) -> Result<OpcodeResolution, OpcodeResolutionError> {
            panic!("Path not trodden by this test")
        }

        fn pedersen(
            &self,
            _initial_witness: &mut WitnessMap,
            _inputs: &[FunctionInput],
            _domain_separator: u32,
            _outputs: &[Witness],
        ) -> Result<OpcodeResolution, OpcodeResolutionError> {
            panic!("Path not trodden by this test")
        }

        fn fixed_base_scalar_mul(
            &self,
            _initial_witness: &mut WitnessMap,
            _input: &FunctionInput,
            _outputs: &[Witness],
        ) -> Result<OpcodeResolution, OpcodeResolutionError> {
            panic!("Path not trodden by this test")
        }
    }

    #[test]
    fn inversion_oracle_equivalence() {
        // Opcodes below describe the following:
        // fn main(x : Field, y : pub Field) {
        //     let z = x + y;
        //     constrain 1/z == Oracle("inverse", x + y);
        // }
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);
        let w_y = Witness(2);
        let w_oracle = Witness(3);
        let w_z = Witness(4);
        let w_z_inverse = Witness(5);
        let opcodes = vec![
            Opcode::Oracle(OracleData {
                name: "invert".into(),
                inputs: vec![Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                    q_c: fe_0,
                }],
                input_values: vec![],
                outputs: vec![w_oracle],
                output_values: vec![],
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
                q_c: fe_0,
            }),
            Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![(fe_1, w_z, w_z_inverse)],
                linear_combinations: vec![],
                q_c: -fe_1,
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
                q_c: fe_0,
            }),
        ];

        let backend = StubbedPwg;

        let mut witness_assignments = BTreeMap::from([
            (Witness(1), FieldElement::from(2u128)),
            (Witness(2), FieldElement::from(3u128)),
        ])
        .into();
        let mut blocks = Blocks::default();
        let solver_status = pwg::solve(&backend, &mut witness_assignments, &mut blocks, opcodes)
            .expect("should stall on oracle");
        let PartialWitnessGeneratorStatus::RequiresOracleData { mut required_oracle_data, unsolved_opcodes, .. } = solver_status else {
            panic!("Should require oracle data")
        };
        assert!(unsolved_opcodes.is_empty(), "oracle should be removed");
        assert_eq!(required_oracle_data.len(), 1, "should have an oracle request");
        let mut oracle_data = required_oracle_data.remove(0);

        assert_eq!(oracle_data.input_values.len(), 1, "Should have solved a single input");

        // Filling data request and continue solving
        oracle_data.output_values = vec![oracle_data.input_values.last().unwrap().inverse()];
        let mut next_opcodes_for_solving = vec![Opcode::Oracle(oracle_data)];
        next_opcodes_for_solving.extend_from_slice(&unsolved_opcodes[..]);
        let solver_status =
            pwg::solve(&backend, &mut witness_assignments, &mut blocks, next_opcodes_for_solving)
                .expect("should be solvable");
        assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
    }

    #[test]
    fn inversion_brillig_oracle_equivalence() {
        // Opcodes below describe the following:
        // fn main(x : Field, y : pub Field) {
        //     let z = x + y;
        //     assert( 1/z == Oracle("inverse", x + y) );
        // }
        // Also performs an unrelated equality check
        // just for the sake of testing multiple brillig opcodes.
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);
        let w_y = Witness(2);
        let w_oracle = Witness(3);
        let w_z = Witness(4);
        let w_z_inverse = Witness(5);
        let w_x_plus_y = Witness(6);
        let w_equal_res = Witness(7);

        let equal_opcode = brillig_vm::Opcode::BinaryFieldOp {
            op: BinaryFieldOp::Equals,
            lhs: RegisterIndex::from(0),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(2),
        };

        let brillig_data = Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                    q_c: fe_0,
                }),
                BrilligInputs::Single(Expression::default()), // Input Register 1
            ],
            // This tells the BrilligSolver which witnesses its output registers correspond to
            outputs: vec![
                BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
                BrilligOutputs::Simple(w_oracle),   // Output Register 1
                BrilligOutputs::Simple(w_equal_res), // Output Register 2
            ],
            // stack of foreign call/oracle resolutions, starts empty
            foreign_call_results: vec![],
            bytecode: vec![
                equal_opcode,
                // Oracles are named 'foreign calls' in brillig
                brillig_vm::Opcode::ForeignCall {
                    function: "invert".into(),
                    destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                    inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
                },
            ],
            predicate: None,
        };

        let opcodes = vec![
            Opcode::Brillig(brillig_data),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
                q_c: fe_0,
            }),
            Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![(fe_1, w_z, w_z_inverse)],
                linear_combinations: vec![],
                q_c: -fe_1,
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
                q_c: fe_0,
            }),
        ];

        let backend = StubbedPwg;

        let mut witness_assignments = BTreeMap::from([
            (Witness(1), FieldElement::from(2u128)),
            (Witness(2), FieldElement::from(3u128)),
        ])
        .into();
        let mut blocks = Blocks::default();
        // use the partial witness generation solver with our acir program
        let solver_status = pwg::solve(&backend, &mut witness_assignments, &mut blocks, opcodes)
            .expect("should stall on oracle");
        let PartialWitnessGeneratorStatus::RequiresOracleData { unsolved_opcodes, mut unresolved_brillig_calls, .. } = solver_status else {
            panic!("Should require oracle data")
        };

        assert_eq!(unsolved_opcodes.len(), 0, "brillig should have been removed");
        assert_eq!(unresolved_brillig_calls.len(), 1, "should have a brillig oracle request");

        let foreign_call = unresolved_brillig_calls.remove(0);
        assert_eq!(
            foreign_call.foreign_call_wait_info.inputs.len(),
            1,
            "Should be waiting for a single input"
        );
        // As caller of VM, need to resolve foreign calls
        let foreign_call_result = vec![Value::from(
            foreign_call.foreign_call_wait_info.inputs[0][0].to_field().inverse(),
        )];
        // Alter Brillig oracle opcode with foreign call resolution
        let brillig: Brillig = foreign_call.resolve(foreign_call_result.into());
        let mut next_opcodes_for_solving = vec![Opcode::Brillig(brillig)];
        next_opcodes_for_solving.extend_from_slice(&unsolved_opcodes[..]);
        // After filling data request, continue solving
        let solver_status =
            pwg::solve(&backend, &mut witness_assignments, &mut blocks, next_opcodes_for_solving)
                .expect("should not stall on oracle");
        assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
    }

    #[test]
    fn double_inversion_brillig_oracle() {
        // Opcodes below describe the following:
        // fn main(x : Field, y : pub Field) {
        //     let z = x + y;
        //     let ij = i + j;
        //     assert( 1/z == Oracle("inverse", x + y) );
        //     assert( 1/ij == Oracle("inverse", i + j) );
        // }
        // Also performs an unrelated equality check
        // just for the sake of testing multiple brillig opcodes.
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);
        let w_y = Witness(2);
        let w_oracle = Witness(3);
        let w_z = Witness(4);
        let w_z_inverse = Witness(5);
        let w_x_plus_y = Witness(6);
        let w_equal_res = Witness(7);
        let w_i = Witness(8);
        let w_j = Witness(9);
        let w_ij_oracle = Witness(10);
        let w_i_plus_j = Witness(11);

        let equal_opcode = brillig_vm::Opcode::BinaryFieldOp {
            op: BinaryFieldOp::Equals,
            lhs: RegisterIndex::from(0),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(4),
        };

        let brillig_data = Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                    q_c: fe_0,
                }),
                BrilligInputs::Single(Expression::default()), // Input Register 1
                BrilligInputs::Single(Expression {
                    // Input Register 2
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_i), (fe_1, w_j)],
                    q_c: fe_0,
                }),
            ],
            outputs: vec![
                BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
                BrilligOutputs::Simple(w_oracle),   // Output Register 1
                BrilligOutputs::Simple(w_i_plus_j), // Output Register 2 - from input
                BrilligOutputs::Simple(w_ij_oracle), // Output Register 3
                BrilligOutputs::Simple(w_equal_res), // Output Register 4
            ],
            // stack of foreign call/oracle resolutions, starts empty
            foreign_call_results: vec![],
            bytecode: vec![
                equal_opcode,
                // Oracles are named 'foreign calls' in brillig
                brillig_vm::Opcode::ForeignCall {
                    function: "invert".into(),
                    destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                    inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
                },
                brillig_vm::Opcode::ForeignCall {
                    function: "invert".into(),
                    destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(3))],
                    inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(2))],
                },
            ],
            predicate: None,
        };

        let opcodes = vec![
            Opcode::Brillig(brillig_data),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
                q_c: fe_0,
            }),
            Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![(fe_1, w_z, w_z_inverse)],
                linear_combinations: vec![],
                q_c: -fe_1,
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
                q_c: fe_0,
            }),
        ];

        let backend = StubbedPwg;

        let mut witness_assignments = BTreeMap::from([
            (Witness(1), FieldElement::from(2u128)),
            (Witness(2), FieldElement::from(3u128)),
            (Witness(8), FieldElement::from(5u128)),
            (Witness(9), FieldElement::from(10u128)),
        ])
        .into();
        let mut blocks = Blocks::default();
        // use the partial witness generation solver with our acir program
        let solver_status = pwg::solve(&backend, &mut witness_assignments, &mut blocks, opcodes)
            .expect("should stall on oracle");
        let PartialWitnessGeneratorStatus::RequiresOracleData { unsolved_opcodes, mut unresolved_brillig_calls, .. } = solver_status else {
            panic!("Should require oracle data")
        };

        assert_eq!(unsolved_opcodes.len(), 0, "brillig should have been removed");
        assert_eq!(unresolved_brillig_calls.len(), 1, "should have a brillig oracle request");

        let foreign_call = unresolved_brillig_calls.remove(0);
        assert_eq!(
            foreign_call.foreign_call_wait_info.inputs.len(),
            1,
            "Should be waiting for a single input"
        );

        let x_plus_y_inverse =
            foreign_call.foreign_call_wait_info.inputs[0][0].to_field().inverse();
        // Alter Brillig oracle opcode
        let brillig: Brillig = foreign_call.resolve(vec![Value::from(x_plus_y_inverse)].into());

        let mut next_opcodes_for_solving = vec![Opcode::Brillig(brillig)];
        next_opcodes_for_solving.extend_from_slice(&unsolved_opcodes[..]);
        // After filling data request, continue solving
        let solver_status =
            pwg::solve(&backend, &mut witness_assignments, &mut blocks, next_opcodes_for_solving)
                .expect("should stall on oracle");
        let PartialWitnessGeneratorStatus::RequiresOracleData { unsolved_opcodes, mut unresolved_brillig_calls, .. } = solver_status else {
            panic!("Should require oracle data")
        };

        assert!(unsolved_opcodes.is_empty(), "should be fully solved");
        assert_eq!(unresolved_brillig_calls.len(), 1, "should have no unresolved oracles");

        let foreign_call = unresolved_brillig_calls.remove(0);
        assert_eq!(
            foreign_call.foreign_call_wait_info.inputs.len(),
            1,
            "Should be waiting for a single input"
        );

        let i_plus_j_inverse =
            foreign_call.foreign_call_wait_info.inputs[0][0].to_field().inverse();
        assert_ne!(x_plus_y_inverse, i_plus_j_inverse);
        // Alter Brillig oracle opcode
        let brillig = foreign_call.resolve(vec![Value::from(i_plus_j_inverse)].into());

        let mut next_opcodes_for_solving = vec![Opcode::Brillig(brillig)];
        next_opcodes_for_solving.extend_from_slice(&unsolved_opcodes[..]);

        // After filling data request, continue solving
        let solver_status =
            pwg::solve(&backend, &mut witness_assignments, &mut blocks, next_opcodes_for_solving)
                .expect("should not stall on oracle");
        assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
    }

    #[test]
    fn brillig_oracle_predicate() {
        // Opcodes below describe the following:
        // fn main(x : Field, y : pub Field, cond: bool) {
        //     let z = x + y;
        //     let z_inverse = 1/z
        //     if cond {
        //         assert( z_inverse == Oracle("inverse", x + y) );
        //     }
        // }
        let fe_0 = FieldElement::zero();
        let fe_1 = FieldElement::one();
        let w_x = Witness(1);
        let w_y = Witness(2);
        let w_oracle = Witness(3);
        let w_z = Witness(4);
        let w_z_inverse = Witness(5);
        let w_x_plus_y = Witness(6);
        let w_equal_res = Witness(7);
        let w_lt_res = Witness(8);

        let equal_opcode = brillig_vm::Opcode::BinaryFieldOp {
            op: BinaryFieldOp::Equals,
            lhs: RegisterIndex::from(0),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(2),
        };

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                    q_c: fe_0,
                }),
                BrilligInputs::Single(Expression::default()),
            ],
            outputs: vec![
                BrilligOutputs::Simple(w_x_plus_y),
                BrilligOutputs::Simple(w_oracle),
                BrilligOutputs::Simple(w_equal_res),
                BrilligOutputs::Simple(w_lt_res),
            ],
            bytecode: vec![
                equal_opcode,
                // Oracles are named 'foreign calls' in brillig
                brillig_vm::Opcode::ForeignCall {
                    function: "invert".into(),
                    destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                    inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
                },
            ],
            predicate: Some(Expression::default()),
            // oracle results
            foreign_call_results: vec![],
        });

        let opcodes = vec![
            brillig_opcode,
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
                q_c: fe_0,
            }),
            Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![(fe_1, w_z, w_z_inverse)],
                linear_combinations: vec![],
                q_c: -fe_1,
            }),
        ];

        let backend = StubbedPwg;

        let mut witness_assignments = BTreeMap::from([
            (Witness(1), FieldElement::from(2u128)),
            (Witness(2), FieldElement::from(3u128)),
        ])
        .into();
        let mut blocks = Blocks::default();
        let solver_status = pwg::solve(&backend, &mut witness_assignments, &mut blocks, opcodes)
            .expect("should not stall on oracle");
        assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
    }
}
