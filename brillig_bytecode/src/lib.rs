//! ACVM is capable of running brillig-bytecode
//! This bytecode is run in the traditional sense
//! and allows one to do non-determinism.
//! This is a generalization over the fixed directives
//! that we have in ACVM.

mod opcodes;
mod registers;
mod value;

pub use opcodes::{BinaryFieldOp, BinaryIntOp, RegisterValueOrArray};
pub use opcodes::{Comparison, Opcode};
pub use registers::{RegisterIndex, Registers};
use serde::{Deserialize, Serialize};
pub use value::Typ;
pub use value::Value;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VMStatus {
    Finished,
    InProgress,
    Failure,
    ForeignCallWait {
        /// Interpreted by simulator context
        function: String,
        /// Input values
        inputs: Vec<Value>,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct ForeignCallResult {
    /// Resolved foreign call values
    pub values: Vec<Value>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VM {
    registers: Registers,
    program_counter: usize,
    foreign_call_counter: usize,
    foreign_call_results: Vec<ForeignCallResult>,
    bytecode: Vec<Opcode>,
    status: VMStatus,
    memory: Vec<Value>,
    call_stack: Vec<Value>,
}

impl VM {
    pub fn new(
        inputs: Registers,
        memory: Vec<Value>,
        bytecode: Vec<Opcode>,
        foreign_call_results: Vec<ForeignCallResult>,
    ) -> VM {
        Self {
            registers: inputs,
            program_counter: 0,
            foreign_call_counter: 0,
            foreign_call_results,
            bytecode,
            status: VMStatus::InProgress,
            memory,
            call_stack: Vec::new(),
        }
    }
/// Returns the current status of the VM.
    fn status(&mut self, status: VMStatus) -> VMStatus {
        self.status = status.clone();
        status
    }
/// Sets the current status of the VM to `finished`.
/// Indicating that the VM has completed execution.
    fn finish(&mut self) -> VMStatus {
        self.status(VMStatus::Finished)
    }

    /// Sets the status of the VM to `ForeignCallWait`.
    /// Indicating that the VM is no waiting for a foreign call to be resolved.
    fn wait_for_foreign_call(&mut self, function: String, inputs: Vec<Value>) -> VMStatus {
        self.status(VMStatus::ForeignCallWait { function, inputs })
    }
/// Sets the current status of the VM to `fail`.
/// Indicating that the VM encoutered a `Trap` Opcode
/// or an invalid state.
    fn fail(&mut self, error_msg: &str) -> VMStatus {
        self.status(VMStatus::Failure);
        // TODO(AD): Proper error handling
        println!("Brillig error: {}", error_msg);
        VMStatus::Failure
    }

    /// Loop over the bytecode and update the program counter
    pub fn process_opcodes(&mut self) -> VMStatus {
        while !matches!(
            self.process_opcode(),
            VMStatus::Finished | VMStatus::Failure | VMStatus::ForeignCallWait { .. }
        ) {}
        self.status.clone()
    }
/// Returns all of the registers in the VM.
    pub fn get_registers(&self) -> &Registers {
        &self.registers
    }

    pub fn get_memory(&self) -> &Vec<Value> {
        &self.memory
    }

    /// Process a single opcode and modify the program counter.
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
                let condition_value = self.registers.get(*condition);
                if !condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::JumpIfNot { condition, location: destination } => {
                let condition_value = self.registers.get(*condition);
                if condition_value.is_zero() {
                    return self.set_program_counter(*destination);
                }
                self.increment_program_counter()
            }
            Opcode::Return => {
                if let Some(register) = self.call_stack.pop() {
                    self.set_program_counter(register.to_usize())
                } else {
                    self.fail("return opcode hit, but callstack already empty")
                }
            }
            Opcode::ForeignCall { function, destination, input } => {
                if self.foreign_call_counter >= self.foreign_call_results.len() {
                    let resolved_inputs = self.resolve_foreign_call_input(*input);
                    return self.wait_for_foreign_call(function.clone(), resolved_inputs);
                }

                let ForeignCallResult { values } =
                    &self.foreign_call_results[self.foreign_call_counter];
                match destination {
                    RegisterValueOrArray::RegisterIndex(index) => {
                        assert_eq!(
                            values.len(),
                            1,
                            "Function result size does not match brillig bytecode"
                        );
                        self.registers.set(*index, values[0])
                    }
                    RegisterValueOrArray::HeapArray(index, size) => {
                        let destination_value = self.registers.get(*index);
                        assert_eq!(
                            values.len(),
                            *size,
                            "Function result size does not match brillig bytecode"
                        );
                        for (i, value) in values.iter().enumerate() {
                            self.memory[destination_value.to_usize() + i] = *value;
                        }
                    }
                }
                self.foreign_call_counter += 1;
                self.increment_program_counter()
            }
            Opcode::Mov { destination: destination_register, source: source_register } => {
                let source_value = self.registers.get(*source_register);
                self.registers.set(*destination_register, source_value);
                self.increment_program_counter()
            }
            Opcode::Trap => self.fail("explicit trap hit in brillig"),
            Opcode::Stop => self.finish(),
            Opcode::Load { destination: destination_register, source_pointer } => {
                // Convert our source_pointer to a usize
                let source = self.registers.get(*source_pointer);
                // Use our usize source index to lookup the value in memory
                let value = &self.memory[source.to_usize()];
                self.registers.set(*destination_register, *value);
                self.increment_program_counter()
            }
            Opcode::Store { destination_pointer, source: source_register } => {
                // Convert our destination_pointer to a usize
                let destination = self.registers.get(*destination_pointer);
                // Use our usize destination index to set the value in memory
                self.memory[destination.to_usize()] = self.registers.get(*source_register);
                self.increment_program_counter()
            }
            Opcode::Call { location } => {
                // Push a return location
                self.call_stack.push(Value::from(self.program_counter + 1));
                self.set_program_counter(*location)
            }
            Opcode::Const { destination, value } => {
                self.registers.set(*destination, *value);
                self.increment_program_counter()
            }
        }
    }
/// Returns the current value of the program counter.
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
            self.status = VMStatus::Finished;
        }
        self.status.clone()
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

    fn resolve_foreign_call_input(&self, input: RegisterValueOrArray) -> Vec<Value> {
        match input {
            RegisterValueOrArray::RegisterIndex(index) => vec![self.registers.get(index)],
            RegisterValueOrArray::HeapArray(index, size) => {
                let start = self.registers.get(index);
                self.memory[start.to_usize()..(start.to_usize() + size)].to_vec()
            }
        }
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
}

pub struct VMOutputState {
    pub registers: Registers,
    pub program_counter: usize,
    pub foreign_call_results: Vec<ForeignCallResult>,
    pub status: VMStatus,
    pub memory: Vec<Value>,
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
        let mut vm = VM::new(input_registers, vec![], vec![opcode], vec![]);

        // Process a single VM opcode
        //
        // After processing a single opcode, we should have
        // the vm status as finished since there is only one opcode
        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished);

        // The register at index `2` should have the value of 3 since we had an
        // add opcode
        let VM { registers, .. } = vm;
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
        opcodes.push(Opcode::JumpIf { condition: RegisterIndex(2), location: 3 });

        let mut vm = VM::new(Registers { inner: registers }, vec![], opcodes, vec![]);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let output_cmp_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(output_cmp_value, Value::from(true));

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::InProgress);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished);
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

        let jump_if_not_opcode = Opcode::JumpIfNot { condition: RegisterIndex(2), location: 1 };

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
            vec![],
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
        let VM { registers, .. } = vm;
        let output_value = registers.get(RegisterIndex(2));
        assert_eq!(output_value, Value::from(false));
    }

    #[test]
    fn mov_opcode() {
        let input_registers =
            Registers::load(vec![Value::from(1u128), Value::from(2u128), Value::from(3u128)]);

        let mov_opcode = Opcode::Mov { destination: RegisterIndex(2), source: RegisterIndex(0) };

        let mut vm = VM::new(input_registers, vec![], vec![mov_opcode], vec![]);

        let status = vm.process_opcode();
        assert_eq!(status, VMStatus::Finished);

        let VM { registers, .. } = vm;

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
            vec![],
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
        assert_eq!(status, VMStatus::Finished);

        let lte_value = vm.registers.get(RegisterIndex(2));
        assert_eq!(lte_value, Value::from(true));
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
                Opcode::Const { destination: r_i, value: 0u128.into() },
                // len = memory.len() (approximation)
                Opcode::Const { destination: r_len, value: Value::from(memory.len() as u128) },
            ];
            let loop_body = [
                // *i = i
                Opcode::Store { destination_pointer: r_i, source: r_i },
                // tmp = 1
                Opcode::Const { destination: r_tmp, value: 1u128.into() },
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
                Opcode::JumpIf { condition: r_tmp, location: start.len() },
            ];
            let vm = brillig_execute_and_get_vm(memory, [&start[..], &loop_body[..]].concat());
            vm.memory
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
                Opcode::Const { destination: r_sum, value: 0u128.into() },
                // i = 0
                Opcode::Const { destination: r_i, value: 0u128.into() },
                // len = array.len() (approximation)
                Opcode::Const { destination: r_len, value: Value::from(memory.len() as u128) },
            ];
            let loop_body = [
                // tmp = *i
                Opcode::Load { destination: r_tmp, source_pointer: r_i },
                // sum = sum + tmp
                Opcode::BinaryIntOp {
                    destination: r_sum,
                    lhs: r_sum,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: 32,
                },
                // tmp = 1
                Opcode::Const { destination: r_tmp, value: 1u128.into() },
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
                Opcode::JumpIf { condition: r_tmp, location: start.len() },
            ];
            let vm = brillig_execute_and_get_vm(memory, [&start[..], &loop_body[..]].concat());
            vm.registers.get(r_sum)
        }

        assert_eq!(
            brillig_sum_memory(vec![
                Value::from(1u128),
                Value::from(2u128),
                Value::from(3u128),
                Value::from(4u128),
                Value::from(5u128),
            ]),
            Value::from(15u128)
        );
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
                Opcode::Const { destination: r_i, value: 0u128.into() },
                // len = memory.len() (approximation)
                Opcode::Const { destination: r_len, value: Value::from(memory.len() as u128) },
                // call recursive_fn
                Opcode::Call {
                    location: 4, // Call after 'start'
                },
                // end program by jumping to end
                Opcode::Jump { location: 100 },
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
                Opcode::Store { destination_pointer: r_i, source: r_i },
                // tmp = 1
                Opcode::Const { destination: r_tmp, value: 1u128.into() },
                // i = i + 1 (tmp)
                Opcode::BinaryIntOp {
                    destination: r_i,
                    lhs: r_i,
                    op: BinaryIntOp::Add,
                    rhs: r_tmp,
                    bit_size: 32,
                },
                // call recursive_fn
                Opcode::Call { location: start.len() },
                Opcode::Return {},
            ];

            let vm = brillig_execute_and_get_vm(memory, [&start[..], &recursive_fn[..]].concat());
            vm.memory
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

    fn empty_registers() -> Registers {
        Registers::load(vec![Value::from(0u128); 16])
    }
    /// Helper to execute brillig code
    fn brillig_execute_and_get_vm(memory: Vec<Value>, opcodes: Vec<Opcode>) -> VM {
        let mut vm = VM::new(empty_registers(), memory, opcodes, vec![]);
        brillig_execute(&mut vm);
        assert_eq!(vm.call_stack, vec![]);
        vm
    }

    fn brillig_execute(vm: &mut VM) {
        loop {
            let status = vm.process_opcode();
            if matches!(status, VMStatus::Finished | VMStatus::ForeignCallWait { .. }) {
                break;
            }
            assert_eq!(status, VMStatus::InProgress)
        }
    }

    #[test]
    fn foreign_call_opcode_register_result() {
        let r_input = RegisterIndex(0);
        let r_result = RegisterIndex(1);

        let double_program = vec![
            // Load input register with value 5
            Opcode::Const { destination: r_input, value: Value::from(5u128) },
            // Call foreign function "double" with the input register
            Opcode::ForeignCall {
                function: "double".into(),
                destination: RegisterValueOrArray::RegisterIndex(r_result),
                input: RegisterValueOrArray::RegisterIndex(r_input),
            },
        ];

        let mut vm = brillig_execute_and_get_vm(vec![], double_program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "double".into(),
                inputs: vec![Value::from(5u128)]
            }
        );

        // Push result we're waiting for
        vm.foreign_call_results.push(ForeignCallResult {
            values: vec![Value::from(10u128)], // Result of doubling 5u128
        });

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished);

        // Check result register
        let result_value = vm.registers.get(r_result);
        assert_eq!(result_value, Value::from(10u128));

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }
    #[test]
    fn foreign_call_opcode_memory_result() {
        let r_input = RegisterIndex(0);
        let r_output = RegisterIndex(1);

        // Define a simple 2x2 matrix in memory
        let initial_matrix =
            vec![Value::from(1u128), Value::from(2u128), Value::from(3u128), Value::from(4u128)];

        // Transpose of the matrix (but arbitrary for this test, the 'correct value')
        let expected_result =
            vec![Value::from(1u128), Value::from(3u128), Value::from(2u128), Value::from(4u128)];

        let invert_program = vec![
            // input = 0
            Opcode::Const { destination: r_input, value: Value::from(0u128) },
            // output = 0
            Opcode::Const { destination: r_output, value: Value::from(0u128) },
            // *output = matrix_2x2_transpose(*input)
            Opcode::ForeignCall {
                function: "matrix_2x2_transpose".into(),
                destination: RegisterValueOrArray::HeapArray(r_output, initial_matrix.len()),
                input: RegisterValueOrArray::HeapArray(r_input, initial_matrix.len()),
            },
        ];

        let mut vm = brillig_execute_and_get_vm(initial_matrix.clone(), invert_program);

        // Check that VM is waiting
        assert_eq!(
            vm.status,
            VMStatus::ForeignCallWait {
                function: "matrix_2x2_transpose".into(),
                inputs: initial_matrix
            }
        );

        // Push result we're waiting for
        vm.foreign_call_results.push(ForeignCallResult { values: expected_result.clone() });

        // Resume VM
        brillig_execute(&mut vm);

        // Check that VM finished once resumed
        assert_eq!(vm.status, VMStatus::Finished);

        // Check result in memory
        let result_values = vm.memory[0..4].to_vec();
        assert_eq!(result_values, expected_result);

        // Ensure the foreign call counter has been incremented
        assert_eq!(vm.foreign_call_counter, 1);
    }
}
