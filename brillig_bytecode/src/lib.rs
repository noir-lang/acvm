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

use acir_field::FieldElement;
pub use opcodes::RegisterMemIndex;
pub use opcodes::{BinaryOp, Comparison, Opcode, OracleData};
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VM {
    registers: Registers,
    program_counter: usize,
    bytecode: Vec<Opcode>,
    status: VMStatus,
}

impl VM {
    pub fn new(mut inputs: Registers, mut bytecode: Vec<Opcode>) -> VM {
        let last_opcode = bytecode.last().expect("need at least one opcode");

        if let Opcode::Bootstrap { register_allocation_indices } = last_opcode {
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
        let vm =
            Self { registers: inputs, program_counter: 0, bytecode, status: VMStatus::InProgress };
        vm
    }

    /// Loop over the bytecode and update the program counter
    pub fn process_opcodes(mut self) -> (Registers, VMStatus, usize) {
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
                let destination = match destination.to_register_index() {
                    Some(register_index) => register_index.0,
                    _ => return VMStatus::Failure,
                };
                self.set_program_counter(destination)
            }
            Opcode::Intrinsics => todo!(),
            Opcode::Oracle(data) => {
                if data.outputs.len() == data.output_values.len() {
                    for (index, value) in data.outputs.iter().zip(data.output_values.iter()) {
                        self.registers.set(*index, (*value).into())
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
    fn finish(self) -> (Registers, VMStatus, usize) {
        (self.registers, self.status, self.program_counter)
    }
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
    let mut vm = VM::new(input_registers, vec![opcode]);

    // Process a single VM opcode
    //
    // After processing a single opcode, we should have
    // the vm status as halted since there is only one opcode
    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::Halted);

    // The register at index `2` should have the value of 3 since we had an
    // add opcode
    let (registers, _, _) = vm.finish();
    let output_value = registers.get(RegisterMemIndex::Register(RegisterIndex(2)));

    assert_eq!(output_value, Value::from(3u128))
}

#[test]
fn test_jmpif_opcode() {
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

    let mut vm = VM::new(input_registers, vec![equal_cmp_opcode, jump_opcode, jump_if_opcode]);

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
fn test_jmpifnot_opcode() {
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
    let (registers, status, _) = vm.finish();
    let output_value = registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(output_value, Value::from(false));
}

#[test]
fn test_mov_opcode() {
    let input_registers =
        Registers::load(vec![Value::from(1u128), Value::from(2u128), Value::from(3u128)]);

    let mov_opcode = Opcode::Mov {
        destination: RegisterMemIndex::Register(RegisterIndex(2)),
        source: RegisterMemIndex::Register(RegisterIndex(0)),
    };

    let mut vm = VM::new(input_registers, vec![mov_opcode]);

    let status = vm.process_opcode();
    assert_eq!(status, VMStatus::Halted);

    let (registers, status, _) = vm.finish();

    let destination_value = registers.get(RegisterMemIndex::Register(RegisterIndex(2)));
    assert_eq!(destination_value, Value::from(1u128));

    let source_value = registers.get(RegisterMemIndex::Register(RegisterIndex(0)));
    assert_eq!(source_value, Value::from(1u128));
}

#[test]
fn test_cmp_binary_ops() {
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
