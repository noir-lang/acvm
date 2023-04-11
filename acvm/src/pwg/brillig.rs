use std::collections::BTreeMap;

use acir::{
    brillig_bytecode::{
        ArrayHeap, Opcode, OracleData, Registers, Typ, VMOutputState, VMStatus, Value, VM,
    },
    circuit::opcodes::{Brillig, BrilligInputs, BrilligOutputs},
    native_types::Witness,
    FieldElement,
};

use crate::{
    pwg::arithmetic::ArithmeticSolver, OpcodeNotSolvable, OpcodeResolution, OpcodeResolutionError,
};

use super::{directives::insert_witness, get_value};

pub struct BrilligSolver;

impl BrilligSolver {
    pub fn solve(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
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

        // A zero predicate indicates the oracle should be skipped, and its ouputs zeroed.
        if pred_value.is_zero() {
            for output in &brillig.outputs {
                match output {
                    BrilligOutputs::Simple(witness) => {
                        insert_witness(*witness, FieldElement::zero(), initial_witness)?
                    }
                    BrilligOutputs::Array(witness_arr) => {
                        for w in witness_arr {
                            insert_witness(*w, FieldElement::zero(), initial_witness)?
                        }
                    }
                }
            }
            return Ok(OpcodeResolution::Solved);
        }

        // Set input values
        let mut input_register_values: Vec<Value> = Vec::new();
        let mut input_memory: BTreeMap<Value, ArrayHeap> = BTreeMap::new();
        for input in &brillig.inputs {
            match input {
                BrilligInputs::Simple(expr) => {
                    // TODO: switch this to `get_value` and map the err
                    let solve = ArithmeticSolver::evaluate(expr, initial_witness);
                    if let Some(value) = solve.to_const() {
                        input_register_values.push(value.into())
                    } else {
                        break;
                    }
                }
                BrilligInputs::Array(id, expr_arr) => {
                    let id_as_value: Value = Value {
                        typ: Typ::Unsigned { bit_size: 32 },
                        inner: FieldElement::from(*id as u128),
                    };
                    // Push value of the array id as a register
                    input_register_values.push(id_as_value.into());

                    let mut continue_eval = true;
                    let mut array_heap: BTreeMap<usize, Value> = BTreeMap::new();
                    for (i, expr) in expr_arr.into_iter().enumerate() {
                        let solve = ArithmeticSolver::evaluate(expr, initial_witness);
                        if let Some(value) = solve.to_const() {
                            array_heap.insert(i, value.into());
                        } else {
                            continue_eval = false;
                            break;
                        }
                    }
                    input_memory.insert(id_as_value, ArrayHeap { memory_map: array_heap });

                    if !continue_eval {
                        break;
                    }
                }
            }
        }

        if input_register_values.len() != brillig.inputs.len() {
            let jabber_input =
                brillig.inputs.last().expect("Infallible: cannot reach this point if no inputs");
            let expr = match jabber_input {
                BrilligInputs::Simple(expr) => expr,
                BrilligInputs::Array(_, expr_arr) => {
                    expr_arr.last().expect("Infallible: cannot reach this point if no inputs")
                }
            };
            return Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::ExpressionHasTooManyUnknowns(
                expr.clone(),
            )));
        }

        let input_registers = Registers { inner: input_register_values };
        let vm = VM::new(input_registers, input_memory, brillig.bytecode.clone());

        let VMOutputState { registers, program_counter, status, memory } = vm.process_opcodes();

        if status == VMStatus::OracleWait {
            let current_opcode = &brillig.bytecode[program_counter];
            let mut data = match current_opcode.clone() {
                Opcode::Oracle(data) => data,
                _ => {
                    return Err(OpcodeResolutionError::UnexpectedOpcode(
                        "brillig oracle",
                        current_opcode.name(),
                    ))
                }
            };

            let input_values = data
                .clone()
                .inputs
                .into_iter()
                .map(|register_mem_index| registers.get(register_mem_index).inner)
                .collect::<Vec<_>>();
            data.input_values = input_values;

            return Ok(OpcodeResolution::InProgressBrillig(OracleWaitInfo {
                data: data.clone(),
                program_counter,
            }));
        }

        for (output, register_value) in brillig.outputs.iter().zip(registers) {
            match output {
                BrilligOutputs::Simple(witness) => {
                    insert_witness(*witness, register_value.inner, initial_witness)?;
                }
                BrilligOutputs::Array(witness_arr) => {
                    let array = memory[&register_value].memory_map.values();
                    for (witness, value) in witness_arr.iter().zip(array) {
                        insert_witness(*witness, value.inner, initial_witness)?;
                    }
                }
            }
        }

        Ok(OpcodeResolution::Solved)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OracleWaitInfo {
    pub data: OracleData,
    pub program_counter: usize,
}
