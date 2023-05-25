use acir::{
    brillig_vm::{RegisterIndex, Registers, VMStatus, Value, VM},
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
        // Each input represents an expression or array of expressions to evaluate.
        // Iterate over each input and evaluate the expression(s) associated with it.
        // Push the results into registers and/or memory.
        for input in &brillig.inputs {
            match input {
                BrilligInputs::Single(expr) => {
                    // TODO: switch this to `get_value` and map the err
                    let solve = ArithmeticSolver::evaluate(expr, initial_witness);
                    if let Some(value) = solve.to_const() {
                        // If the expression evaluates to a constant, great!
                        // Add it to the register set and proceed.
                        input_register_values.push(value.into())
                    } else {
                        // If the expression has too many unknowns to resolve to a const,
                        // it can't be added to a register and we can't execute the
                        // Brillig bytecode on these inputs just yet.
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
                            // If the expression evaluates to a constant, great!
                            // Add it to the register set and proceed.
                            input_memory.push(value.into());
                        } else {
                            // If the expression has too many unknowns to resolve to a const,
                            // it can't be added to a register and we can't execute the
                            // Brillig bytecode on these inputs just yet.
                            continue_eval = false;
                            break;
                        }
                    }

                    // If an array value is missing, exit the input solver and do not insert the array id as an input register
                    if !continue_eval {
                        break;
                    }

                    // Push value of the array pointer as a register
                    input_register_values.push(Value::from(memory_pointer));
                }
            }
        }

        // Number of brillig inputs should match number of registers here.
        // Otherwise some expression (the last one processed before the above loop exited)
        // was not solvable, in which case we do not proceed with Brillig VM execution.
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

        // Instantiate a Brillig VM given the solved input registers and memory
        // along with the Brillig bytecode, and any present foreign call results.
        let input_registers = Registers { inner: input_register_values };
        let mut vm = VM::new(
            input_registers,
            input_memory,
            brillig.bytecode.clone(),
            brillig.foreign_call_results.clone(),
        );

        // Run the Brillig VM on these inputs, bytecode, etc!
        let vm_status = vm.process_opcodes();

        // Check the status of the Brillig VM.
        // It may be finished, in-progress, failed, or may be waiting for results of a foreign call.
        // Return the "resolution" to the caller who may choose to make subsequent calls
        // (when it gets foreign call results for example).
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

#[derive(Debug, PartialEq, Clone)]
pub struct ForeignCallWaitInfo {
    pub function: String,
    pub inputs: Vec<Value>,
}
