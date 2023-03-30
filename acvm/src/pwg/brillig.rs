use std::collections::BTreeMap;

use acir::{
    brillig_bytecode::{Opcode, OracleData, RegisterMemIndex, Registers, VMStatus, Value, VM},
    circuit::opcodes::Brillig,
    native_types::Witness,
    FieldElement,
};

use crate::{
    pwg::arithmetic::ArithmeticSolver, OpcodeNotSolvable, OpcodeResolution, OpcodeResolutionError,
};

use super::directives::insert_witness;

pub struct BrilligSolver;

impl BrilligSolver {
    pub fn solve(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        brillig: &mut Brillig,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        // Set input values
        let mut input_register_values: Vec<Value> = Vec::new();
        for expr in &brillig.inputs {
            // Break from setting the inputs values if unable to solve the arithmetic expression inputs
            // TODO: switch this to `get_value` and map the err
            let solve = ArithmeticSolver::evaluate(expr, initial_witness);
            if let Some(value) = solve.to_const() {
                input_register_values.push(value.into())
            } else {
                break;
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
        let vm = VM::new(input_registers, brillig.bytecode.clone());

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

        let output_register_values: Vec<FieldElement> =
            output_registers.clone().inner.into_iter().map(|v| v.inner).collect::<Vec<_>>();

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
