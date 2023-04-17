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
pub use opcodes::RegisterMemIndex;
pub use opcodes::{BinaryOp, Comparison, Opcode, OracleData, OracleInput};
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
                Registers::load(vec![Value { typ: Typ::Field, inner: FieldElement::from(0u128) }]);

            for i in 0..register_allocation_indices.len() {
                let register_index = register_allocation_indices[i];
                let register_value = inputs.get(RegisterMemIndex::Register(RegisterIndex(i)));
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
        };
        vm
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
            Opcode::BinaryOp { op, lhs, rhs, result, result_type } => {
                self.process_binary_op(*op, *lhs, *rhs, *result, *result_type);
                self.increment_program_counter()
            }
            Opcode::JMP { destination } => self.set_program_counter(*destination),
            Opcode::JMPIF { condition, destination } => {
                // Check if condition is true
                // We use 0 to mean false and any other value to mean true
                let condition_value = self.registers.get(*condition);
                if !condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::JMPIFNOT { condition, destination } => {
                let condition_value = self.registers.get(*condition);
                if condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::Call { destination } => {
                let register = self.registers.get(*destination);
                let label = usize::try_from(
                    register.inner.try_to_u64().expect("register does not fit into u64"),
                )
                .expect("register does not fit into usize");
                self.set_program_counter(label)
            }
            Opcode::Intrinsics => todo!(),
            Opcode::Oracle(data) => {
                if data.output_values.len() == 1 {
                    self.registers.set(data.output, data.output_values[0].into());
                } else if data.output_values.len() > 1 {
                    let register = self.registers.get(RegisterMemIndex::Register(data.output));
                    let heap = &mut self.memory.entry(register).or_default().memory_map;
                    for (i, value) in data.output_values.iter().enumerate() {
                        heap.insert(i, (*value).into());
                    }
                } else {
                    self.status = VMStatus::OracleWait;
                    return VMStatus::OracleWait;
                }
                self.increment_program_counter()
            }
            Opcode::Mov { destination, source } => {
                let source_value = self.registers.get(*source);

                match destination {
                    RegisterMemIndex::Register(dest_index) => {
                        self.registers.set(*dest_index, source_value)
                    }
                    _ => return VMStatus::Failure, // TODO: add variants to VMStatus::Failure for more informed failures
                }

                self.increment_program_counter()
            }
            Opcode::Trap => VMStatus::Failure,
            Opcode::Bootstrap { .. } => unreachable!(
                "should only be at end of opcodes and popped off when initializing the vm"
            ),
            Opcode::Stop => VMStatus::Halted,
            Opcode::Load { destination, array_id_reg, index } => {
                let array_id = self.registers.get(*array_id_reg);
                let array = &self.memory[&array_id];
                match destination {
                    RegisterMemIndex::Register(dest_index) => {
                        let index_value = self.registers.get(*index);
                        let index_usize = usize::try_from(
                            index_value.inner.try_to_u64().expect("register does not fit into u64"),
                        )
                        .expect("register does not fit into usize");
                        self.registers.set(*dest_index, array.memory_map[&index_usize]);
                    }
                    _ => return VMStatus::Failure, // TODO: add variants to VMStatus::Failure for more informed failures
                }
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
    fn process_binary_op(
        &mut self,
        op: BinaryOp,
        lhs: RegisterMemIndex,
        rhs: RegisterMemIndex,
        result: RegisterIndex,
        result_type: Typ,
    ) {
        let lhs_value = self.registers.get(lhs);
        let rhs_value = self.registers.get(rhs);

        let result_value = op.function()(lhs_value, rhs_value);

        self.registers.set(result, result_value)
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

#[test]
fn add_single_step_smoke() {
    // Load values into registers and initialize the registers that
    // will be used during bytecode processing
    let input_registers =
        Registers::load(vec![Value::from(1u128), Value::from(2u128), Value::from(0u128)]);

    // Add opcode to add the value in register `0` and `1`
    // and place the output in register `2`
    let opcode = Opcode::BinaryOp {
        op: BinaryOp::Add,
        lhs: RegisterMemIndex::Register(RegisterIndex(0)),
        rhs: RegisterMemIndex::Register(RegisterIndex(1)),
        result: RegisterIndex(2),
        result_type: Typ::Field,
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
    let output_value = registers.get(RegisterMemIndex::Register(RegisterIndex(2)));

    assert_eq!(output_value, Value::from(3u128))
}

#[test]
fn jmpif_opcode() {
    let input_registers =
        Registers::load(vec![Value::from(2u128), Value::from(2u128), Value::from(0u128)]);

    let equal_cmp_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Eq),
        lhs: RegisterMemIndex::Register(RegisterIndex(0)),
        rhs: RegisterMemIndex::Register(RegisterIndex(1)),
        result: RegisterIndex(2),
    };

    let jump_opcode = Opcode::JMP { destination: 2 };

    let jump_if_opcode =
        Opcode::JMPIF { condition: RegisterMemIndex::Register(RegisterIndex(2)), destination: 3 };

    let mut vm = VM::new(
        input_registers,
        BTreeMap::new(),
        vec![equal_cmp_opcode, jump_opcode, jump_if_opcode],
    );

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    let output_cmp_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
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

    let not_equal_cmp_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Eq),
        lhs: RegisterMemIndex::Register(RegisterIndex(0)),
        rhs: RegisterMemIndex::Register(RegisterIndex(1)),
        result: RegisterIndex(2),
    };

    let jump_opcode = Opcode::JMP { destination: 2 };

    let jump_if_not_opcode = Opcode::JMPIFNOT {
        condition: RegisterMemIndex::Register(RegisterIndex(2)),
        destination: 1,
    };

    let add_opcode = Opcode::BinaryOp {
        op: BinaryOp::Add,
        lhs: RegisterMemIndex::Register(RegisterIndex(0)),
        rhs: RegisterMemIndex::Register(RegisterIndex(1)),
        result: RegisterIndex(2),
        result_type: Typ::Field,
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

    let output_cmp_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(output_cmp_value, Value::from(false));

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::Failure);

    // The register at index `2` should have not changed as we jumped over the add opcode
    let VMOutputState { registers, .. } = vm.finish();
    let output_value = registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(output_value, Value::from(false));
}

#[test]
fn mov_opcode() {
    let input_registers =
        Registers::load(vec![Value::from(1u128), Value::from(2u128), Value::from(3u128)]);

    let mov_opcode = Opcode::Mov {
        destination: RegisterMemIndex::Register(RegisterIndex(2)),
        source: RegisterMemIndex::Register(RegisterIndex(0)),
    };

    let mut vm = VM::new(input_registers, BTreeMap::new(), vec![mov_opcode]);

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::Halted);

    let VMOutputState { registers, .. } = vm.finish();

    let destination_value = registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(destination_value, Value::from(1u128));

    let source_value = registers.get(RegisterMemIndex::Register(RegisterIndex(0)));
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

    let equal_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Eq),
        lhs: RegisterMemIndex::Register(RegisterIndex(0)),
        rhs: RegisterMemIndex::Register(RegisterIndex(1)),
        result: RegisterIndex(2),
    };

    let not_equal_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Eq),
        lhs: RegisterMemIndex::Register(RegisterIndex(0)),
        rhs: RegisterMemIndex::Register(RegisterIndex(3)),
        result: RegisterIndex(2),
    };

    let less_than_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Lt),
        lhs: RegisterMemIndex::Register(RegisterIndex(3)),
        rhs: RegisterMemIndex::Register(RegisterIndex(4)),
        result: RegisterIndex(2),
    };

    let less_than_equal_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Lte),
        lhs: RegisterMemIndex::Register(RegisterIndex(3)),
        rhs: RegisterMemIndex::Register(RegisterIndex(4)),
        result: RegisterIndex(2),
    };

    let mut vm = VM::new(
        input_registers,
        BTreeMap::new(),
        vec![equal_opcode, not_equal_opcode, less_than_opcode, less_than_equal_opcode],
    );

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    let output_eq_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(output_eq_value, Value::from(true));

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    let output_neq_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(output_neq_value, Value::from(false));

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    let lt_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(lt_value, Value::from(true));

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::Halted);

    let lte_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
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

    let equal_cmp_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Eq),
        lhs: RegisterMemIndex::Register(RegisterIndex(0)),
        rhs: RegisterMemIndex::Register(RegisterIndex(1)),
        result: RegisterIndex(2),
    };

    let jump_opcode = Opcode::JMP { destination: 3 };

    let jump_if_opcode =
        Opcode::JMPIF { condition: RegisterMemIndex::Register(RegisterIndex(2)), destination: 10 };

    let load_opcode = Opcode::Load {
        destination: RegisterMemIndex::Register(RegisterIndex(4)),
        array_id_reg: RegisterMemIndex::Register(RegisterIndex(3)),
        index: RegisterMemIndex::Register(RegisterIndex(2)),
    };

    let mem_equal_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Eq),
        lhs: RegisterMemIndex::Register(RegisterIndex(4)),
        rhs: RegisterMemIndex::Register(RegisterIndex(5)),
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

    let output_cmp_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(output_cmp_value, Value::from(true));

    // load_opcode
    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    let output_cmp_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(4)));
    assert_eq!(output_cmp_value, Value::from(6u128));

    // jump_opcode
    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    // mem_equal_opcode
    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    let output_cmp_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(6)));
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
    ]);

    let equal_cmp_opcode = Opcode::BinaryOp {
        result_type: Typ::Field,
        op: BinaryOp::Cmp(Comparison::Eq),
        lhs: RegisterMemIndex::Register(RegisterIndex(0)),
        rhs: RegisterMemIndex::Register(RegisterIndex(1)),
        result: RegisterIndex(2),
    };

    let jump_opcode = Opcode::JMP { destination: 3 };

    let jump_if_opcode =
        Opcode::JMPIF { condition: RegisterMemIndex::Register(RegisterIndex(2)), destination: 10 };

    let store_opcode = Opcode::Store {
        source: RegisterMemIndex::Register(RegisterIndex(2)),
        array_id_reg: RegisterMemIndex::Register(RegisterIndex(3)),
        index: RegisterMemIndex::Constant(FieldElement::from(3_u128)),
    };

    let mut initial_memory = BTreeMap::new();
    let initial_heap = ArrayHeap {
        memory_map: BTreeMap::from([(0 as usize, Value::from(5u128)), (1, Value::from(6u128))]),
    };
    initial_memory.insert(Value::from(5u128), initial_heap);

    let mut vm = VM::new(
        input_registers,
        initial_memory,
        vec![equal_cmp_opcode, store_opcode, jump_opcode, jump_if_opcode],
    );

    // equal_cmp_opcode
    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::InProgress);

    let output_cmp_value = vm.registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(output_cmp_value, Value::from(true));

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

    let oracle_input =
        OracleInput { register_mem_index: RegisterMemIndex::Register(RegisterIndex(0)), length: 0 };

    let mut oracle_data = OracleData {
        name: "get_notes".to_owned(),
        inputs: vec![oracle_input],
        input_values: vec![],
        output: RegisterIndex(3),
        output_values: vec![],
    };

    let oracle_opcode = Opcode::Oracle(oracle_data.clone());

    let initial_memory = BTreeMap::new();

    let vm = VM::new(input_registers.clone(), initial_memory, vec![oracle_opcode]);

    let output_state = vm.process_opcodes();
    assert_eq!(output_state.status, VMStatus::OracleWait);

    let mut input_values = Vec::new();
    for oracle_input in oracle_data.clone().inputs {
        if oracle_input.length == 0 {
            let x = output_state.registers.get(oracle_input.register_mem_index).inner;
            input_values.push(x);
        } else {
            let array_id = output_state.registers.get(oracle_input.register_mem_index);
            let array = output_state.memory[&array_id].clone();
            let heap_fields =
                array.memory_map.into_values().map(|value| value.inner).collect::<Vec<_>>();
            input_values.extend(heap_fields);
        }
    }

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

    let oracle_input =
        OracleInput { register_mem_index: RegisterMemIndex::Register(RegisterIndex(3)), length: 2 };

    let mut oracle_data = OracleData {
        name: "call_private_function_oracle".to_owned(),
        inputs: vec![oracle_input.clone()],
        input_values: vec![],
        output: RegisterIndex(6),
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

    let mut input_values = Vec::new();
    for oracle_input in oracle_data.clone().inputs {
        if oracle_input.length == 0 {
            let x = output_state.registers.get(oracle_input.register_mem_index).inner;
            input_values.push(x);
        } else {
            let array_id = output_state.registers.get(oracle_input.register_mem_index);
            let array = output_state.memory[&array_id].clone();
            let heap_fields =
                array.memory_map.into_values().map(|value| value.inner).collect::<Vec<_>>();
            input_values.extend(heap_fields);
        }
    }
    assert_eq!(input_values.len(), oracle_input.length);

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
