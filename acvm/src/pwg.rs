// Re-usable methods that backends can use to implement their PWG

use crate::{Language, PartialWitnessGenerator};
use acir::{
    circuit::opcodes::{BlackBoxFuncCall, Opcode, OracleData},
    native_types::{Expression, Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};

use self::{
    arithmetic::ArithmeticSolver, block::Blocks, directives::solve_directives, oracle::OracleSolver,
};

use thiserror::Error;

// arithmetic
pub mod arithmetic;
// Directives
pub mod directives;
// black box functions
mod blackbox;
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
}

pub fn solve(
    backend: &impl PartialWitnessGenerator,
    initial_witness: &mut WitnessMap,
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

// Inserts `value` into the initial witness map
// under the key of `witness`.
// Returns an error, if there was already a value in the map
// which does not match the value that one is about to insert
fn insert_value(
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
        !matches!(opcode, Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AES { .. }) | Opcode::Block(_))
    }

    match language {
        Language::R1CS => r1cs_is_supported,
        Language::PLONKCSat { .. } => plonk_is_supported,
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acir::{
        circuit::{
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
        fn aes(
            &self,
            _initial_witness: &mut WitnessMap,
            _inputs: &[FunctionInput],
            _outputs: &[Witness],
        ) -> Result<OpcodeResolution, OpcodeResolutionError> {
            panic!("Path not trodden by this test")
        }

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

        fn verify_proof(
            &self,
            _initial_witness: &mut WitnessMap,
            _key: &[FunctionInput],
            _proof: &[FunctionInput],
            _public_inputs: &[FunctionInput],
            _input_aggregation_object: &[FunctionInput],
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
        let PartialWitnessGeneratorStatus::RequiresOracleData { mut required_oracle_data, unsolved_opcodes } = solver_status else {
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
}
