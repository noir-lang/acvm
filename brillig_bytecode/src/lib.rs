// ACVM is capable of running brillig-bytecode
// This bytecode is ran in the traditional sense
// and allows one to do non-determinism.
// This is a generalization over the fixed directives
// that we have in ACVM.

mod opcodes;
mod registers;
mod value;

pub use opcodes::{BinaryOp, Opcode, UnaryOp};
pub use registers::{RegisterIndex, Registers};
pub use value::Value;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VMStatus {
    Halted,
    InProgress,
}

pub struct VM {
    registers: Registers,
    program_counter: usize,
    bytecode: Vec<Opcode>,
    status: VMStatus,
}

impl VM {
    pub fn new(inputs: Registers, bytecode: Vec<Opcode>) -> VM {
        Self { registers: inputs, program_counter: 0, bytecode, status: VMStatus::InProgress }
    }

    /// Loop over the bytecode and update the program counter
    pub fn process_opcodes(mut self) -> Registers {
        while self.process_opcode() != VMStatus::Halted {}
        self.finish()
    }
    // Process a single opcode and modify the program counter
    pub fn process_opcode(&mut self) -> VMStatus {
        let opcode = &self.bytecode[self.program_counter];
        match opcode {
            Opcode::BinaryOp { op, lhs, rhs, result } => {
                self.process_binary_op(*op, *lhs, *rhs, *result);
                self.increment_program_counter()
            }
            Opcode::UnaryOp { op, input, result } => {
                self.process_unary_op(*op, *input, *result);
                self.increment_program_counter()
            }
            Opcode::JMP { destination } => self.set_program_counter(destination.inner()),
            Opcode::JMPIF { condition, destination } => {
                // Check if condition is true
                // We use 0 to mean false and any other value to mean true

                let condition_value = self.registers.get(*condition);
                if !condition_value.is_zero() {
                    return self.set_program_counter(destination.inner());
                }
                self.status
            }
            Opcode::Call => todo!(),
            Opcode::Intrinsics => todo!(),
            Opcode::Oracle => todo!(),
            Opcode::Store => todo!(),
            Opcode::Load => todo!(),
        }
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
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
    ) {
        let lhs_value = self.registers.get(lhs);
        let rhs_value = self.registers.get(rhs);

        let result_value = op.function()(lhs_value, rhs_value);

        self.registers.set(result, result_value)
    }

    /// Process a unary operation.
    /// This method will not modify the program counter.
    fn process_unary_op(&mut self, op: UnaryOp, input: RegisterIndex, result: RegisterIndex) {
        let input_value = self.registers.get(input);

        let result_value = op.function()(input_value);

        self.registers.set(result, result_value)
    }
    /// Returns the state of the registers.
    /// This consumes ownership of the VM and is conventionally
    /// called when all of the bytecode has been processed.
    fn finish(self) -> Registers {
        self.registers
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
        lhs: RegisterIndex(0),
        rhs: RegisterIndex(1),
        result: RegisterIndex(2),
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
    let registers = vm.finish();
    let output_value = registers.get(RegisterIndex(2));

    assert_eq!(output_value, Value::from(3u128))
}
