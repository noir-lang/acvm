use std::collections::BTreeMap;

use acir::{
    brillig_bytecode::{Opcode, OracleData, Registers, Typ, VMStatus, Value, VM},
    circuit::opcodes::{Brillig, JabberingIn, JabberingOut},
    native_types::Witness,
    FieldElement,
};
use k256::elliptic_curve::Field;

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
                    JabberingOut::Simple(witness) => {
                        insert_witness(*witness, FieldElement::zero(), initial_witness)?
                    }
                    JabberingOut::Array(witness_arr) => {
                        //todo, the
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
        let mut input_memory: BTreeMap<u32, Vec<Value>> = BTreeMap::new();
        for input in &brillig.inputs {
            match input {
                JabberingIn::Simple(epxr) => {
                    // TODO: switch this to `get_value` and map the err
                    let solve = ArithmeticSolver::evaluate(expr, initial_witness);
                    if let Some(value) = solve.to_const() {
                        input_register_values.push(value.into())
                    } else {
                        break;
                    }
                }
                JabberingIn::Array(id, expr_arr) => {
                    let id_as_value: Value =
                        Value { typ: Typ::ArrayId, inner: FieldElement::from(id as u128) };
                    // Push value of the array id as a register
                    input_register_values.push(id_as_value.into());

                    let continue_eval = true;
                    let array_heap: Vec<Value> = Vec::new();
                    for expr in expr_arr {
                        let solve = ArithmeticSolver::evaluate(expr, initial_witness);
                        if let Some(value) = solve.to_const() {
                            array_heap.push(value.into())
                        } else {
                            continue_eval = false;
                            break;
                        }
                    }
                    input_memory.insert(id, array_heap);

                    if !continue_eval {
                        break;
                    }
                }
            }
        }

        if input_register_values.len() != brillig.inputs.len() {
            return Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::ExpressionHasTooManyUnknowns(
                brillig
                    .inputs
                    .last()
                    .expect("Infallible: cannot reach this point if no inputs")
                    .clone(),
            )));
        }

        let input_registers = Registers { inner: input_register_values };
        let vm = VM::new(input_registers, input_memory, brillig.bytecode.clone());

        let (output_registers, status, pc) = vm.process_opcodes();

        if status == VMStatus::OracleWait {
            let current_opcode = &brillig.bytecode[pc];
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
                .map(|register_mem_index| output_registers.get(register_mem_index).inner)
                .collect::<Vec<_>>();
            data.input_values = input_values;

            return Ok(OpcodeResolution::InProgressBrillig(OracleWaitInfo {
                data: data.clone(),
                program_counter: pc,
            }));
        }

        let output_register_values: Vec<FieldElement> = output_registers
            .clone()
            .inner
            .into_iter()
            .map(|v| match v.typ {
                Typ::ArrayId => vm.load_array(&v),
                _ => v.inner,
            })
            .collect::<Vec<_>>();

        for (witness, value) in brillig.outputs.iter().zip(output_register_values) {
            insert_witness(*witness, value, initial_witness)?;
        }

        Ok(OpcodeResolution::Solved)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OracleWaitInfo {
    pub data: OracleData,
    pub program_counter: usize,
}
