// ACVM is capable of running brillig-bytecode
// This bytecode is ran in the traditional sense
// and allows one to do non-determinism.
// This is a generalization over the fixed directives
// that we have in ACVM.

mod memory;
mod opcodes;
mod registers;
mod value;


use acir_field::FieldElement;
pub use opcodes::{BinaryFieldOp, BinaryIntOp};
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
pub struct HeapArray {
    pub array: Vec<Value>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VM {
    registers: Registers,
    program_counter: usize,
    bytecode: Vec<Opcode>,
    status: VMStatus,
    memory: Vec<Value>,
    call_stack: Vec<Value>,
}

impl VM {
    pub fn new(
        mut inputs: Registers,
        memory: Vec<Value>,
        bytecode: Vec<Opcode>,
        register_allocation_indices: Option<Vec<u32>>
    ) -> VM {
        if let Some(register_allocation_indices) = register_allocation_indices {
            // TODO: might have to handle arrays in bootstrap to be correct
            // TODO(AD): simplify this all to be done before calling VM.new()
            let mut registers_modified =
                Registers::load(vec![Value::from(0u128)]);
    
            for i in 0..register_allocation_indices.len() {
                let register_index = register_allocation_indices[i];
                let register_value = inputs.get(RegisterIndex(i));
                registers_modified.set(RegisterIndex(register_index as usize), register_value)
            }
    
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
            Opcode::BinaryFieldOp { op, lhs, rhs, destination: result } => {
                self.process_binary_field_op(*op, *lhs, *rhs, *result);
                self.increment_program_counter()
            }
            Opcode::BinaryIntOp { op, bit_size, lhs, rhs, destination: result } => {
                self.process_binary_int_op(*op, *bit_size, *lhs, *rhs, *result);
                self.increment_program_counter()
            }
            Opcode::Jump { location: destination } => self.set_program_counter(*destination),
            Opcode::JumpIf { condition, location: destination } => {
                // Check if condition is true
                // We use 0 to mean false and any other value to mean true
                let condition_value = self.get(condition);
                if !condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::JumpIfNot { condition, location: destination } => {
                let condition_value = self.get(condition);
                if condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::Return => {
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
                        }
                    }
                }

                self.increment_program_counter()
            }
            Opcode::Mov { destination: destination_register, source: source_register } => {
                let source_value = self.get(source_register);
                self.registers.set(*destination_register, source_value);
                self.increment_program_counter()
            }
            Opcode::Trap => self.fail(),
            Opcode::Stop => self.halt(),
            Opcode::Load { destination: destination_register, source_pointer } => {
                // Convert our source_pointer to a usize
                let source = self.get(source_pointer);
                let source_usize = usize::try_from(
                    source.inner.try_to_u64().expect("register does not fit into u64"),
                ).expect("register does not fit into usize");
                // Use our usize source index to lookup the value in memory
                let value = &self.memory[source_usize];
                self.registers.set(*destination_register, *value);
                self.increment_program_counter()
            }
            Opcode::Store { destination_pointer, source: source_register } => {
                // Convert our destination_pointer to a usize
                let destination = self.get(destination_pointer);
                let destination_usize = usize::try_from(
                    destination.inner.try_to_u64().expect("register does not fit into u64"),
                ).expect("register does not fit into usize");
                // Use our usize destination index to set the value in memory
                self.memory[destination_usize] = self.get(source_register);
                self.increment_program_counter()
            }
            Opcode::Call { location } => {
                // Push a return location
                self.call_stack.push(Value::from(self.program_counter + 1));
                self.set_program_counter(*location)
            }
            Opcode::Const { destination, value } => {
                self.registers.set(*destination, Value::from(*value));
                self.increment_program_counter()
            }
        }
    }

    /// Get the value of a register
    fn get(&self, register: &RegisterIndex) -> Value {
        self.registers.get(*register)
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
        op: BinaryIntOp,
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
        op: BinaryFieldOp,
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
    pub memory: Vec<Value>,
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
            op: BinaryIntOp::Add,
            bit_size: 2,
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            destination: RegisterIndex(2),
        };

        // Start VM
        let mut vm = VM::new(input_registers, vec![], vec![opcode], None);

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
        let mut registers = vec![];
        let mut opcodes = vec![];
    
        let lhs = {
            registers.push(Value::from(2u128));
            RegisterIndex(registers.len() - 1)
        };
    
        let rhs = {
            registers.push(Value::from(2u128));
            RegisterIndex(registers.len() - 1)
        };
    
        let destination = {
            registers.push(Value::from(0u128));
            RegisterIndex(registers.len() - 1)
        };
    
        let equal_cmp_opcode = Opcode::BinaryIntOp {
            op: BinaryIntOp::Cmp(Comparison::Eq),
            bit_size: 1,
            lhs,
            rhs,
            destination,
        };
        opcodes.push(equal_cmp_opcode);
        opcodes.push(Opcode::Jump { location: 2 });
        opcodes.push(Opcode::JumpIf {
            condition: RegisterIndex(2),
            location: 3,
        });
    
        let mut vm = VM::new(
            Registers { inner: registers },
            vec![],
            opcodes,
            None,
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
            op: BinaryFieldOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            destination: RegisterIndex(2),
        };

        let jump_opcode = Opcode::Jump { location: 2 };

        let jump_if_not_opcode = Opcode::JumpIfNot {
            condition: RegisterIndex(2),
            location: 1,
        };

        let add_opcode = Opcode::BinaryFieldOp {
            op: BinaryFieldOp::Add,
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            destination: RegisterIndex(2),
        };

        let mut vm = VM::new(
            input_registers,
            vec![],
            vec![jump_opcode, trap_opcode, not_equal_cmp_opcode, jump_if_not_opcode, add_opcode],
            None
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

        let mut vm = VM::new(input_registers, vec![], vec![mov_opcode], None);

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
            op: BinaryIntOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(1),
            destination: RegisterIndex(2),
        };

        let not_equal_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryIntOp::Cmp(Comparison::Eq),
            lhs: RegisterIndex(0),
            rhs: RegisterIndex(3),
            destination: RegisterIndex(2),
        };

        let less_than_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryIntOp::Cmp(Comparison::Lt),
            lhs: RegisterIndex(3),
            rhs: RegisterIndex(4),
            destination: RegisterIndex(2),
        };

        let less_than_equal_opcode = Opcode::BinaryIntOp {
            bit_size: 1,
            op: BinaryIntOp::Cmp(Comparison::Lte),
            lhs: RegisterIndex(3),
            rhs: RegisterIndex(4),
            destination: RegisterIndex(2),
        };

        let mut vm = VM::new(
            input_registers,
            vec![],
            vec![equal_opcode, not_equal_opcode, less_than_opcode, less_than_equal_opcode],
            None,
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
    fn store_opcode() {
        /// Brillig code for the following:
        ///     let mut i = 0;
        ///     let len = memory.len();
        ///     while i < len {
        ///         memory[i] = i as Value;
        ///         i += 1;
        ///     }
        fn brillig_write_memory(memory: Vec<Value>) -> Vec<Value> {
            let r_i = RegisterIndex(0);
            let r_len = RegisterIndex(1);
            let r_tmp = RegisterIndex(2);
            let start = [
                // i = 0
                Opcode::Const {
                    destination: r_i, 
                    value: 0u128.into(),
                },
                // len = memory.len() (approximation)
                Opcode::Const {
                    destination: r_len,
                    value: FieldElement::from(memory.len() as u128),
                },
            ];
            let loop_body = [
                // *i = i
                Opcode::Store { 
                    destination_pointer: r_i,
                    source: r_i,
                },
                // tmp = 1
                Opcode::Const {
                    destination: r_tmp,
                    value: 1u128.into(),
                },
                // i = i + 1 (tmp)
                Opcode::BinaryIntOp {
                    destination: r_i,
                    lhs: r_i,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: 32,
                },
                // tmp = i < len
                Opcode::BinaryIntOp {
                    destination: r_tmp,
                    lhs: r_i,
                    op: BinaryIntOp::Cmp(Comparison::Lt), 
                    rhs: r_len,
                    bit_size: 32,
                },
                // if tmp != 0 goto loop_body
                Opcode::JumpIf {
                    condition: r_tmp,
                    location: start.len()
                },
            ];
            let vm = brillig_execute(memory, [&start[..], &loop_body[..]].concat());
            vm.memory.clone()
        }
    
        let memory = brillig_write_memory(vec![Value::from(0u128); 5]);
        let expected = vec![
            Value::from(0u128),
            Value::from(1u128),
            Value::from(2u128),
            Value::from(3u128),
            Value::from(4u128),
        ];
        assert_eq!(memory, expected);
    
        let memory = brillig_write_memory(vec![Value::from(0u128); 1024]);
        let expected: Vec<Value> = (0..1024).map(|i| Value::from(i as u128)).collect();
        assert_eq!(memory, expected);
    }

    #[test]
    fn load_opcode() {
        /// Brillig code for the following:
        ///     let mut sum = 0;
        ///     let mut i = 0;
        ///     let len = memory.len();
        ///     while i < len {
        ///         sum += memory[i];
        ///         i += 1;
        ///     }
        fn brillig_sum_memory(memory: Vec<Value>) -> Value {
            let r_i = RegisterIndex(0);
            let r_len = RegisterIndex(1);
            let r_sum = RegisterIndex(2);
            let r_tmp = RegisterIndex(3);
            let start = [
                // sum = 0
                Opcode::Const {
                    destination: r_sum,
                    value: 0u128.into(),
                },
                // i = 0
                Opcode::Const {
                    destination: r_i, 
                    value: 0u128.into(),
                },
                // len = array.len() (approximation)
                Opcode::Const {
                    destination: r_len,
                    value: FieldElement::from(memory.len() as u128),
                },
            ];
            let loop_body = [
                // tmp = *i
                Opcode::Load { 
                    destination: r_tmp,
                    source_pointer: r_i
                },
                // sum = sum + tmp
                Opcode::BinaryIntOp {
                    destination: r_sum,
                    lhs: r_sum,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: 32,
                },
                // tmp = 1
                Opcode::Const {
                    destination: r_tmp,
                    value: 1u128.into(),
                },
                // i = i + 1 (tmp)
                Opcode::BinaryIntOp {
                    destination: r_i,
                    lhs: r_i,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: 32,
                },
                // tmp = i < len
                Opcode::BinaryIntOp {
                    destination: r_tmp,
                    lhs: r_i,
                    op: BinaryIntOp::Cmp(Comparison::Lt), 
                    rhs: r_len,
                    bit_size: 32,
                },
                // if tmp != 0 goto loop_body
                Opcode::JumpIf {
                    condition: r_tmp,
                    location: start.len()
                },
            ];
            let vm = brillig_execute(memory, [&start[..], &loop_body[..]].concat());
            vm.registers.get(r_sum)
        }

        assert_eq!(brillig_sum_memory(vec![
            Value::from(1u128),
            Value::from(2u128),
            Value::from(3u128),
            Value::from(4u128),
            Value::from(5u128),
        ]), Value::from(15u128));
        assert_eq!(brillig_sum_memory(vec![Value::from(1u128); 1024]), Value::from(1024u128));
    }

    #[test]
    fn call_and_return_opcodes() {
        /// Brillig code for the following recursive function:
        ///     fn recursive_write(i: u128, len: u128) {
        ///         if len <= i {
        ///             return;
        ///         }
        ///         memory[i as usize] = i as Value;
        ///         recursive_write(memory, i + 1, len);
        ///     }
        /// Note we represent a 100% in-register optimized form in brillig
        fn brillig_recursive_write_memory(memory: Vec<Value>) -> Vec<Value> {
            let r_i = RegisterIndex(0);
            let r_len = RegisterIndex(1);
            let r_tmp = RegisterIndex(2);
    
            let start = [
                // i = 0
                Opcode::Const {
                    destination: r_i,
                    value: 0u128.into(),
                },
                // len = memory.len() (approximation)
                Opcode::Const {
                    destination: r_len,
                    value: FieldElement::from(memory.len() as u128),
                },
            ];
    
            let recursive_fn = [
                // tmp = len <= i
                Opcode::BinaryIntOp {
                    destination: r_tmp,
                    lhs: r_len,
                    op: BinaryIntOp::Cmp(Comparison::Lte),
                    rhs: r_i,
                    bit_size: 32,
                },
                // if !tmp, goto end
                Opcode::JumpIf {
                    condition: r_tmp,
                    location: start.len() + 6, // 7 ops in recursive_fn, go to 'Return'
                },
                // *i = i
                Opcode::Store {
                    destination_pointer: r_i,
                    source: r_i,
                },
                // tmp = 1
                Opcode::Const {
                    destination: r_tmp,
                    value: 1u128.into(),
                },
                // i = i + 1 (tmp)
                Opcode::BinaryIntOp {
                    destination: r_i,
                    lhs: r_i,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: 32,
                },
                // call recursive_fn
                Opcode::Jump {
                    location: start.len(),
                },
                Opcode::Return {}
            ];
    
            let vm = brillig_execute(memory, [&start[..], &recursive_fn[..]].concat());
            vm.memory.clone()
        }
    
        let memory = brillig_recursive_write_memory(vec![Value::from(0u128); 5]);
        let expected = vec![
            Value::from(0u128),
            Value::from(1u128),
            Value::from(2u128),
            Value::from(3u128),
            Value::from(4u128),
        ];
        assert_eq!(memory, expected);
    
        let memory = brillig_recursive_write_memory(vec![Value::from(0u128); 1024]);
        let expected: Vec<Value> = (0..1024).map(|i| Value::from(i as u128)).collect();
        assert_eq!(memory, expected);
    }
    
    /// Helper to execute brillig code
    fn brillig_execute(memory: Vec<Value>, opcodes: Vec<Opcode>) -> VM {
        let mut vm = VM::new(
            Registers { inner: vec![Value::from(0u128); 16]},
            memory,
            opcodes,
            None,
        );
        loop {
            let status = vm.process_opcode();
            if status == VMStatus::Halted {
                break;
            }
        }
        vm
    }
}
