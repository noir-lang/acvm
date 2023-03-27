use acir::FieldElement;

use crate::{
    value::{Typ, Value},
    RegisterIndex,
};

#[derive(Debug, Clone, Copy)]
pub struct ArrayIndex {
    pointer: usize,
    index: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterMemIndex {
    Register(RegisterIndex),
    Memory(ArrayIndex),
    Value(FieldElement),
}

#[derive(Debug, Clone)]
pub enum Opcode {
    /// Takes the values in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryOp {
        result_type: Typ,
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
    Oracle {
        inputs: Vec<RegisterIndex>,
        destination: Vec<RegisterIndex>,
    },
    Mov {
        destination: RegisterMemIndex,
        source: RegisterMemIndex,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Cmp(Comparison),
}

#[derive(Debug, Clone, Copy)]
pub enum Comparison {}

impl BinaryOp {
    pub fn function(&self) -> fn(Value, Value) -> Value {
        match self {
            BinaryOp::Add => |a: Value, b: Value| a + b,
            BinaryOp::Sub => |a: Value, b: Value| a - b,
            BinaryOp::Mul => |a: Value, b: Value| a * b,
            BinaryOp::Div => |a: Value, b: Value| a / b,
            BinaryOp::Cmp(_) => todo!(),
        }
    }
}
