use acir::{
    brillig_vm::{Opcode, RegisterIndex, Registers, VMStatus, Value, VM},
    circuit::brillig::{Brillig, BrilligInputs, BrilligOutputs},
    native_types::WitnessMap,
    FieldElement,
};

use crate::{
    pwg::{arithmetic::ArithmeticSolver, OpcodeNotSolvable},
    OpcodeResolution, OpcodeResolutionError,
};

use super::{get_value, insert_value};

pub struct BrilligSolver;

impl BrilligSolver {
    pub fn solve(
        initial_witness: &mut WitnessMap,
        brillig: &mut Brillig,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        // If the predicate is `None`, then we simply return the value 1
        // If the predicate is `Some` but we cannot find a value, then we return stalled
        let pred_value = match &brillig.predicate {
            Some(pred) => get_value(pred, initial_witness),
            None => Ok(FieldElement::one()),
        };
        let pred_value = match pred_value {
            Ok(pred_value) => pred_value,
            Err(OpcodeResolutionError::OpcodeNotSolvable(unsolved)) => {
                return Ok(OpcodeResolution::Stalled(unsolved))
            }
            Err(err) => return Err(err),
        };

        // A zero predicate indicates the oracle should be skipped, and its outputs zeroed.
        if pred_value.is_zero() {
            for output in &brillig.outputs {
                match output {
                    BrilligOutputs::Simple(witness) => {
                        insert_value(witness, FieldElement::zero(), initial_witness)?
                    }
                    BrilligOutputs::Array(witness_arr) => {
                        for w in witness_arr {
                            insert_value(w, FieldElement::zero(), initial_witness)?
                        }
                    }
                }
            }
            return Ok(OpcodeResolution::Solved);
        }

        // Set input values
        let mut input_register_values: Vec<Value> = Vec::new();
        let mut input_memory: Vec<Value> = Vec::new();
        for input in &brillig.inputs {
            match input {
                BrilligInputs::Single(expr) => {
                    // TODO: switch this to `get_value` and map the err
                    let solve = ArithmeticSolver::evaluate(expr, initial_witness);
                    if let Some(value) = solve.to_const() {
                        input_register_values.push(value.into())
                    } else {
                        break;
                    }
                }
                BrilligInputs::Array(expr_arr) => {
                    // Attempt to fetch all array input values
                    let mut continue_eval = true;
                    let memory_pointer = input_memory.len();
                    for expr in expr_arr.iter() {
                        let solve = ArithmeticSolver::evaluate(expr, initial_witness);
                        if let Some(value) = solve.to_const() {
                            input_memory.push(value.into());
                        } else {
                            continue_eval = false;
                            break;
                        }
                    }

                    // If an array value is missing exit the input solver and do not insert the array id as an input register
                    if !continue_eval {
                        break;
                    }

                    // Push value of the array pointer as a register
                    input_register_values.push(Value::from(memory_pointer));
                }
            }
        }

        if input_register_values.len() != brillig.inputs.len() {
            let brillig_input =
                brillig.inputs.last().expect("Infallible: cannot reach this point if no inputs");
            let expr = match brillig_input {
                BrilligInputs::Single(expr) => expr,
                BrilligInputs::Array(expr_arr) => {
                    expr_arr.last().expect("Infallible: cannot reach this point if no inputs")
                }
            };
            return Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::ExpressionHasTooManyUnknowns(
                expr.clone(),
            )));
        }

        let input_registers = Registers { inner: input_register_values };
        let mut vm = VM::new(
            input_registers,
            input_memory,
            brillig.bytecode.clone(),
            brillig.foreign_call_results.clone(),
        );

        let vm_status = vm.process_opcodes();

        let result = match vm_status {
            VMStatus::Finished => {
                for (i, output) in brillig.outputs.iter().enumerate() {
                    let register_value = vm.get_registers().get(RegisterIndex::from(i));
                    match output {
                        BrilligOutputs::Simple(witness) => {
                            insert_value(witness, register_value.to_field(), initial_witness)?;
                        }
                        BrilligOutputs::Array(witness_arr) => {
                            // Treat the register value as a pointer to memory
                            for (i, witness) in witness_arr.iter().enumerate() {
                                let value = &vm.get_memory()[register_value.to_usize() + i];
                                insert_value(witness, value.to_field(), initial_witness)?;
                            }
                        }
                    }
                }
                OpcodeResolution::Solved
            }
            VMStatus::InProgress => unreachable!("Brillig VM has not completed execution"),
            VMStatus::Failure => return Err(OpcodeResolutionError::UnsatisfiedConstrain),
            VMStatus::ForeignCallWait { function, inputs } => {
                OpcodeResolution::InProgressBrillig(ForeignCallWaitInfo { function, inputs })
            }
        };

        Ok(result)
    }
}

/// Encapsulates a request from a VM process that encounters a [foreign call opcode][Opcode::ForeignCall]  
/// where the result of the foreign call has not yet been provided.
///
/// The caller must resolve this opcode externally based upon the information in the request.
#[derive(Debug, PartialEq, Clone)]
pub struct ForeignCallWaitInfo {
    /// An identifier interpreted by the caller process
    pub function: String,
    /// Resolved inputs to a foreign call computed in the previous steps of a Brillig VM process
    pub inputs: Vec<Value>,
}
