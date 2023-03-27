use crate::{value::Value, RegisterIndex};

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    /// Takes the values in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryOp {
        op: BinaryOp,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
    },
    /// Sets the program counter to the value located at `destination`
    /// If the value at condition is non-zero
    JMPIF {
        condition: RegisterIndex,
        destination: RegisterIndex,
    },
    /// Sets the program counter to the value located at `destination`
    JMP {
        destination: RegisterIndex,
    },
    // TODO:This is used to call functions and setup things like
    // TODO execution contexts.
    Call,
    // TODO:These are special functions like sha256
    Intrinsics,
    // TODO:This will be used to get data from an outside source
    Oracle,
    // TODO: This will be used to store a value at a particular index
    RegisterStore,
    // TODO: This will be used to explicitly load a value at a particular index
    RegisterLoad,
    //
    ArrayStore,
    ArrayLoad,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinaryOp {
    pub fn function(&self) -> fn(Value, Value) -> Value {
        match self {
            BinaryOp::Add => |a: Value, b: Value| a + b,
            BinaryOp::Sub => |a: Value, b: Value| a - b,
            BinaryOp::Mul => |a: Value, b: Value| a * b,
            BinaryOp::Div => |a: Value, b: Value| a / b,
        }
    }
}
