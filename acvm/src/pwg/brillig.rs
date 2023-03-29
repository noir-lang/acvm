use std::collections::BTreeMap;

use acir::{circuit::opcodes::Brillig, native_types::Witness, FieldElement};

use crate::{OpcodeNotSolvable, OpcodeResolution, OpcodeResolutionError};

use super::{directives::insert_witness, get_value};

pub struct BrilligSolver;

impl BrilligSolver {
    pub fn solve(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        brillig: &mut Brillig,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        let mut input_register_values: Vec<acir::brillig_bytecode::Value> = Vec::new();
        for expr in &brillig.inputs {
            let expr_value = get_value(expr, initial_witness)?;
            input_register_values.push(expr_value.into())
        }
        let input_registers = acir::brillig_bytecode::Registers { inner: input_register_values };

        let vm = acir::brillig_bytecode::VM::new(input_registers, brillig.bytecode.clone());

        let output_registers = vm.process_opcodes();

        let output_register_values: Vec<FieldElement> =
            output_registers.inner.into_iter().map(|v| v.inner).collect::<Vec<_>>();

        for (witness, value) in brillig.outputs.iter().zip(output_register_values) {
            insert_witness(*witness, value, initial_witness)?;
        }

        Ok(OpcodeResolution::Solved)
    }
}
