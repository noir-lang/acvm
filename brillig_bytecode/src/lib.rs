// ACVM is capable of running brillig-bytecode
// This bytecode is ran in the traditional sense
// and allows one to do non-determinism.
// This is a generalization over the fixed directives
// that we have in ACVM.

mod builder;
mod memory;
mod opcodes;
mod registers;
mod value;

use std::collections::BTreeMap;

use acir_field::FieldElement;
use num_bigint::{BigInt, Sign};
pub use opcodes::BinaryOp;
pub use opcodes::RegisterMemIndex;
pub use opcodes::{Comparison, Opcode, OracleData, OracleInput, OracleOutput};
pub use registers::{RegisterIndex, Registers};
pub use value::Typ;
pub use value::Value;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VMStatus {
    Halted,
    InProgress,
    Failure,
    OracleWait,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ArrayHeap {
    // maps memory address to Value
    pub memory_map: BTreeMap<usize, Value>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VM {
    registers: Registers,
    program_counter: usize,
    bytecode: Vec<Opcode>,
    status: VMStatus,
    memory: BTreeMap<Value, ArrayHeap>,
    call_stack: Vec<Value>,
}

impl VM {
    pub fn new(
        mut inputs: Registers,
        memory: BTreeMap<Value, ArrayHeap>,
        mut bytecode: Vec<Opcode>,
    ) -> VM {
        let last_opcode = bytecode.last().expect("need at least one opcode");

        if let Opcode::Bootstrap { register_allocation_indices } = last_opcode {
            // TODO: might have to handle arrays in bootstrap to be correct
            let mut registers_modified =
                Registers::load(vec![Value { inner: FieldElement::from(0u128) }]);

            for i in 0..register_allocation_indices.len() {
                let register_index = register_allocation_indices[i];
                let register_value = inputs.get(RegisterIndex(i));
                registers_modified.set(RegisterIndex(register_index as usize), register_value)
            }

            bytecode.pop();
            inputs = registers_modified;
        }
        let vm = Self {
            registers: inputs,
            program_counter: 0,
            bytecode,
            status: VMStatus::InProgress,
            memory,
            call_stack: Vec::new(),
        };
        vm
    }

    fn status(&mut self, status: VMStatus) -> VMStatus {
        self.status = status;
        status
    }
    fn halt(&mut self) -> VMStatus {
        self.status(VMStatus::Halted)
    }
    fn wait(&mut self) -> VMStatus {
        self.status(VMStatus::OracleWait)
    }
    fn fail(&mut self) -> VMStatus {
        self.status(VMStatus::Failure)
    }

    /// Loop over the bytecode and update the program counter
    pub fn process_opcodes(mut self) -> VMOutputState {
        while !matches!(
            self.process_opcode(),
            VMStatus::Halted | VMStatus::Failure | VMStatus::OracleWait
        ) {}
        self.finish()
    }
    // Process a single opcode and modify the program counter
    pub fn process_opcode(&mut self) -> VMStatus {
        let opcode = &self.bytecode[self.program_counter];
        match opcode {
            Opcode::BinaryFieldOp { op, lhs, rhs, result } => {
                self.process_binary_field_op(*op, *lhs, *rhs, *result);
                self.increment_program_counter()
            }
            Opcode::BinaryIntOp { op, bit_size, lhs, rhs, result } => {
                self.process_binary_int_op(*op, *bit_size, *lhs, *rhs, *result);
                self.increment_program_counter()
            }
            Opcode::Jump { destination } => self.set_program_counter(*destination),
            Opcode::JumpIf { condition, destination } => {
                // Check if condition is true
                // We use 0 to mean false and any other value to mean true
                let condition_value = self.registers.get(*condition);
                if !condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::JumpIfNot { condition, destination } => {
                let condition_value = self.registers.get(*condition);
                if condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::Call => {
                if let Some(register) = self.call_stack.pop() {
                    let label = usize::try_from(
                        register.inner.try_to_u64().expect("register does not fit into u64"),
                    )
                    .expect("register does not fit into usize");
                    self.set_program_counter(label)
                } else {
                    return self.halt();
                }
            }
            Opcode::Intrinsics => todo!(),
            Opcode::Oracle(data) => {
                let mut num_output_values = 0;
                for oracle_output in data.clone().outputs {
                    match oracle_output {
                        OracleOutput::RegisterIndex(_) => num_output_values += 1,
                        OracleOutput::Array { length, .. } => num_output_values += length,
                    }
                }
                if num_output_values != data.output_values.len() {
                    return self.wait();
                } else {
                    let mut current_value_index = 0;
                    for oracle_output in data.clone().outputs {
                        match oracle_output {
                            OracleOutput::RegisterIndex(index) => {
                                self.registers
                                    .set(index, data.output_values[current_value_index].into());
                                current_value_index += 1
                            }
                            OracleOutput::Array { start, length } => {
                                let array_id = self.registers.get(start);
                                let heap = &mut self.memory.entry(array_id).or_default().memory_map;
                                for (i, value) in data.output_values.iter().enumerate() {
                                    heap.insert(i, (*value).into());
                                }
                                current_value_index += length
                            }
                        }
                    }
                }

                self.increment_program_counter()
            }
            Opcode::Mov { destination, source } => {
                let source_value = self.registers.get(*source);
                self.registers.set(*destination, source_value);
                self.increment_program_counter()
            }
            Opcode::Trap => self.fail(),
            Opcode::Bootstrap { .. } => unreachable!(
                "should only be at end of opcodes and popped off when initializing the vm"
            ),
            Opcode::Stop => self.halt(),
            Opcode::Load { destination, array_id_reg, index } => {
                let array_id = self.registers.get(*array_id_reg);
                let array = &self.memory[&array_id];
                let index_value = self.registers.get(*index);
                let index_usize = usize::try_from(
                    index_value.inner.try_to_u64().expect("register does not fit into u64"),
                ).expect("register does not fit into usize");
                self.registers.set(*destination, array.memory_map[&index_usize]);
                self.increment_program_counter()
            }
            Opcode::Store { source, array_id_reg, index } => {
                let source_value = self.registers.get(*source);
                let array_id = self.registers.get(*array_id_reg);
                let heap = &mut self.memory.entry(array_id).or_default().memory_map;

                let index_value = self.registers.get(*index);
                let index_usize = usize::try_from(
                    index_value.inner.try_to_u64().expect("register does not fit into u64"),
                )
                .expect("register does not fit into usize");
                heap.insert(index_usize, source_value);

                self.increment_program_counter()
            }
            Opcode::PushStack { source } => {
                let register = self.registers.get(*source);
                self.call_stack.push(register);
                self.increment_program_counter()
            }
            Opcode::LoadConst { destination, constant } => {
                self.registers.set(*destination, Value { inner: *constant });
                self.increment_program_counter()
            }
        }
    }

    pub fn program_counter(self) -> usize {
        self.program_counter
    }

    /// Increments the program counter by 1.
    fn increment_program_counter(&mut self) -> VMStatus {
        self.set_program_counter(self.program_counter + 1)
    }

    /// Increments the program counter by `value`.
    /// If the program counter no longer points to an opcode
    /// in the bytecode, then the VMStatus reports halted.
    fn set_program_counter(&mut self, value: usize) -> VMStatus {
        assert!(self.program_counter < self.bytecode.len());
        self.program_counter = value;
        if self.program_counter >= self.bytecode.len() {
            self.status = VMStatus::Halted;
        }
        self.status
    }

    /// Process a binary operation.
    /// This method will not modify the program counter.
    fn process_binary_int_op(
        &mut self,
        op: BinaryOp,
        bit_size: u32,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
    ) {
        let lhs_value = self.registers.get(lhs);
        let rhs_value = self.registers.get(rhs);

        let result_value = op.evaluate_int(lhs_value.to_u128(), rhs_value.to_u128(), bit_size);

        self.registers.set(result, result_value.into());
    }

    /// Process a binary operation.
    /// This method will not modify the program counter.
    fn process_binary_field_op(
        &mut self,
        op: BinaryOp,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
    ) {
        let lhs_value = self.registers.get(lhs);
        let rhs_value = self.registers.get(rhs);

        let result_value = op.evaluate_field(lhs_value.inner, rhs_value.inner);

        self.registers.set(result, result_value.into())
    }

    /// Returns the state of the registers.
    /// This consumes ownership of the VM and is conventionally
    /// called when all of the bytecode has been processed.
    fn finish(self) -> VMOutputState {
        VMOutputState {
            registers: self.registers,
            program_counter: self.program_counter,
            status: self.status,
            memory: self.memory,
        }
    }
}

pub struct VMOutputState {
    pub registers: Registers,
    pub program_counter: usize,
    pub status: VMStatus,
    pub memory: BTreeMap<Value, ArrayHeap>,
}

impl VMOutputState {
    pub fn map_input_values(&self, oracle_data: &OracleData) -> Vec<FieldElement> {
        let mut input_values = vec![];
        for oracle_input in &oracle_data.inputs {
            match oracle_input {
                OracleInput::RegisterIndex(register_index) => {
                    let register = self.registers.get(*register_index);
                    input_values.push(register.inner);
                }
                OracleInput::Array { start, length } => {
                    let array_id = self.registers.get(*start);
                    let array = &self.memory[&array_id];
                    let heap_fields = array.memory_map.values().map(|value| value.inner.clone());

                    assert_eq!(heap_fields.len(), *length);
                    input_values.extend(heap_fields);
                }
            }
        }
        input_values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_single_step_smoke() {
        // Load values into registers and initialize the registers that
        // will be used during bytecode processing
        let input_registers =
            Registers::load(vec![Value::from(1u128), Value::from(2u128), Value::from(0u128)]);

        // Add opcode to add the value in register `0` and `1`
        // and place the output in register `2`
        let opcode = Opcode::BinaryIntOp {
            op: BinaryOp::Add,
            bit_size: 2,
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            result: RegisterIndex(2),
        };

        // Start VM
        let mut vm = VM::new(input_registers, BTreeMap::new(), vec![opcode]);

        // Process a single VM opcode
        //
        // After processing a single opcode, we should have
        // the vm status as halted since there is only one opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Halted);

        // The register at index `2` should have the value of 3 since we had an
        // add opcode
        let VMOutputState { registers, .. } = vm.finish();
        let output_value = registers.get(RegisterIndex(2));

        assert_eq!(output_value, Value::from(3u128))
    }

    #[test]
    fn jmpif_opcode() {
        let input_registers =
            Registers::load(vec![Value::from(2u128), Value::from(2u128), Value::from(0u128)]);

        let equal_cmp_opcode = Opcode::BinaryIntOp {
            op: BinaryOp::Cmp(Comparison::Eq),
            bit_size: 1,
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            result: RegisterIndex(2),
        };

        let jump_opcode = Opcode::Jump { destination: 2 };

        let jump_if_opcode = Opcode::JumpIf {
            condition: RegisterIndex(2),
            destination: 3,
        };

        let mut vm = VM::new(
            input_registers,
            BTreeMap::new(),
            vec![equal_cmp_opcode, jump_opcode, jump_if_opcode],
        );

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(output_cmp_value, Value::from(true));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Halted);

        vm.finish();
    }

    #[test]
    fn jmpifnot_opcode() {
        let input_registers =
            Registers::load(vec![Value::from(1u128), Value::from(2u128), Value::from(0u128)]);

        let trap_opcode = Opcode::Trap;

        let not_equal_cmp_opcode = Opcode::BinaryFieldOp {
            op: BinaryOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            result: RegisterIndex(2),
        };

        let jump_opcode = Opcode::Jump { destination: 2 };

        let jump_if_not_opcode = Opcode::JumpIfNot {
            condition: RegisterIndex(2),
            destination: 1,
        };

        let add_opcode = Opcode::BinaryFieldOp {
            op: BinaryOp::Add,
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            result: RegisterIndex(2),
        };

        let mut vm = VM::new(
            input_registers,
            BTreeMap::new(),
            vec![jump_opcode, trap_opcode, not_equal_cmp_opcode, jump_if_not_opcode, add_opcode],
        );

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(output_cmp_value, Value::from(false));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Failure);

        // The register at index `2` should have not changed as we jumped over the add opcode
        let VMOutputState { registers, .. } = vm.finish();
        let output_value = registers.get(RegisterIndex(2));
        assert_eq!(output_value, Value::from(false));
    }

    #[test]
    fn mov_opcode() {
        let input_registers =
            Registers::load(vec![Value::from(1u128), Value::from(2u128), Value::from(3u128)]);

        let mov_opcode = Opcode::Mov {
            destination: RegisterIndex(2),
            source: RegisterIndex(0),
        };

        let mut vm = VM::new(input_registers, BTreeMap::new(), vec![mov_opcode]);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Halted);

        let VMOutputState { registers, .. } = vm.finish();

        let destination_value = registers.get(RegisterIndex(2));
        assert_eq!(destination_value, Value::from(1u128));

        let source_value = registers.get(RegisterIndex(0));
        assert_eq!(source_value, Value::from(1u128));
    }

    #[test]
    fn cmp_binary_ops() {
        let input_registers = Registers::load(vec![
            Value::from(2u128),
            Value::from(2u128),
            Value::from(0u128),
            Value::from(5u128),
            Value::from(6u128),
        ]);

        let equal_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            result: RegisterIndex(2),
        };

        let not_equal_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(3),
            result: RegisterIndex(2),
        };

        let less_than_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryOp::Cmp(Comparison::Lt),
            lhs: RegisterIndex(3),
            rhs: RegisterIndex(4),
            result: RegisterIndex(2),
        };

        let less_than_equal_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryOp::Cmp(Comparison::Lte),
            lhs: RegisterIndex(3),
            rhs: RegisterIndex(4),
            result: RegisterIndex(2),
        };

        let mut vm = VM::new(
            input_registers,
            BTreeMap::new(),
            vec![equal_opcode, not_equal_opcode, less_than_opcode, less_than_equal_opcode],
        );

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_eq_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(output_eq_value, Value::from(true));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_neq_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(output_neq_value, Value::from(false));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let lt_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(lt_value, Value::from(true));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Halted);

        let lte_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(lte_value, Value::from(true));

        vm.finish();
    }

    #[test]
    fn load_opcode() {
        let input_registers = Registers::load(vec![
            Value::from(2u128),
            Value::from(2u128),
            Value::from(0u128),
            Value::from(5u128),
            Value::from(0u128),
            Value::from(6u128),
            Value::from(0u128),
        ]);

        let equal_cmp_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            result: RegisterIndex(2),
        };

        let jump_opcode = Opcode::Jump { destination: 3 };

        let jump_if_opcode = Opcode::JumpIf {
            condition: RegisterIndex(2),
            destination: 10,
        };

        let load_opcode = Opcode::Load {
            destination: RegisterIndex(4),
            array_id_reg: RegisterIndex(3),
            index: RegisterIndex(2),
        };

        let mem_equal_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(4),
            rhs: RegisterIndex(5),
            result: RegisterIndex(6),
        };

        let mut initial_memory = BTreeMap::new();
        let initial_heap = ArrayHeap {
            memory_map: BTreeMap::from([(0 as usize, Value::from(5u128)), (1, Value::from(6u128))]),
        };
        initial_memory.insert(Value::from(5u128), initial_heap);

        let mut vm = VM::new(
            input_registers,
            initial_memory,
            vec![equal_cmp_opcode, load_opcode, jump_opcode, mem_equal_opcode, jump_if_opcode],
        );

        // equal_cmp_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(output_cmp_value, Value::from(true));

        // load_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.registers.get(RegisterIndex(4));
        assert_eq!(output_cmp_value, Value::from(6u128));

        // jump_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        // mem_equal_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.registers.get(RegisterIndex(6));
        assert_eq!(output_cmp_value, Value::from(true));

        // jump_if_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Halted);

        vm.finish();
    }

    #[test]
    fn store_opcode() {
        let input_registers = Registers::load(vec![
            Value::from(2u128),
            Value::from(2u128),
            Value::from(0u128),
            Value::from(5u128),
            Value::from(0u128),
            Value::from(6u128),
            Value::from(0u128),
            Value::from(0u128),
        ]);

        let equal_cmp_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            result: RegisterIndex(2),
        };

        let jump_opcode = Opcode::Jump { destination: 4 };

        let jump_if_opcode = Opcode::JumpIf {
            condition: RegisterIndex(2),
            destination: 11,
        };

        let load_const_opcode = Opcode::LoadConst {
            destination: RegisterIndex(7),
            constant: 3_u128.into()
        };

        let store_opcode = Opcode::Store {
            source: RegisterIndex(2),
            array_id_reg: RegisterIndex(3),
            index: RegisterIndex(7),
        };

        let mut initial_memory = BTreeMap::new();
        let initial_heap = ArrayHeap {
            memory_map: BTreeMap::from([(0 as usize, Value::from(5u128)), (1, Value::from(6u128))]),
        };
        initial_memory.insert(Value::from(5u128), initial_heap);

        let mut vm = VM::new(
            input_registers,
            initial_memory,
            vec![equal_cmp_opcode, load_const_opcode, store_opcode, jump_opcode, jump_if_opcode],
        );

        // equal_cmp_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(output_cmp_value, Value::from(true));

        // load_const_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        // store_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let mem_array = vm.memory[&Value::from(5u128)].clone();
        assert_eq!(mem_array.memory_map[&3], Value::from(true));

        // jump_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        // jump_if_opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Halted);

        vm.finish();
    }

    #[test]
    fn oracle_array_output() {
        use crate::opcodes::OracleInput;

        let input_registers = Registers::load(vec![
            Value::from(2u128),
            Value::from(2u128),
            Value::from(0u128),
            Value::from(5u128),
            Value::from(0u128),
            Value::from(6u128),
            Value::from(0u128),
        ]);

        let oracle_input = OracleInput::RegisterIndex(RegisterIndex(0));
        let oracle_output =
            OracleOutput::Array { start: RegisterIndex(3), length: 2 };

        let mut oracle_data = OracleData {
            name: "get_notes".to_owned(),
            inputs: vec![oracle_input],
            input_values: vec![],
            outputs: vec![oracle_output],
            output_values: vec![],
        };

        let oracle_opcode = Opcode::Oracle(oracle_data.clone());

        let initial_memory = BTreeMap::new();

        let vm = VM::new(input_registers.clone(), initial_memory, vec![oracle_opcode]);

        let output_state = vm.process_opcodes();
        assert_eq!(output_state.status, VMStatus::OracleWait);

        let input_values = output_state.map_input_values(&oracle_data);

        oracle_data.input_values = input_values;
        oracle_data.output_values = vec![FieldElement::from(10_u128), FieldElement::from(2_u128)];
        let updated_oracle_opcode = Opcode::Oracle(oracle_data);

        let vm = VM::new(input_registers, output_state.memory, vec![updated_oracle_opcode]);
        let output_state = vm.process_opcodes();
        assert_eq!(output_state.status, VMStatus::Halted);

        let mem_array = output_state.memory[&Value::from(5u128)].clone();
        assert_eq!(mem_array.memory_map[&0], Value::from(10_u128));
        assert_eq!(mem_array.memory_map[&1], Value::from(2_u128));
    }

    #[test]
    fn oracle_array_input() {
        use crate::opcodes::OracleInput;

        let input_registers = Registers::load(vec![
            Value::from(2u128),
            Value::from(2u128),
            Value::from(0u128),
            Value::from(5u128),
            Value::from(0u128),
            Value::from(6u128),
            Value::from(0u128),
        ]);

        let expected_length = 2;
        let oracle_input = OracleInput::Array {
            start: RegisterIndex(3),
            length: expected_length,
        };
        let oracle_output = OracleOutput::RegisterIndex(RegisterIndex(6));

        let mut oracle_data = OracleData {
            name: "call_private_function_oracle".to_owned(),
            inputs: vec![oracle_input.clone()],
            input_values: vec![],
            outputs: vec![oracle_output],
            output_values: vec![],
        };

        let oracle_opcode = Opcode::Oracle(oracle_data.clone());

        let mut initial_memory = BTreeMap::new();
        let initial_heap = ArrayHeap {
            memory_map: BTreeMap::from([(0 as usize, Value::from(5u128)), (1, Value::from(6u128))]),
        };
        initial_memory.insert(Value::from(5u128), initial_heap);

        let vm = VM::new(input_registers.clone(), initial_memory, vec![oracle_opcode]);

        let output_state = vm.process_opcodes();
        assert_eq!(output_state.status, VMStatus::OracleWait);

        let input_values = output_state.map_input_values(&oracle_data);
        assert_eq!(input_values.len(), expected_length);

        oracle_data.input_values = input_values;
        oracle_data.output_values = vec![FieldElement::from(5_u128)];
        let updated_oracle_opcode = Opcode::Oracle(oracle_data);

        let vm = VM::new(input_registers, output_state.memory, vec![updated_oracle_opcode]);
        let output_state = vm.process_opcodes();
        assert_eq!(output_state.status, VMStatus::Halted);

        let mem_array = output_state.memory[&Value::from(5u128)].clone();
        assert_eq!(mem_array.memory_map[&0], Value::from(5_u128));
        assert_eq!(mem_array.memory_map[&1], Value::from(6_u128));
    }
}
