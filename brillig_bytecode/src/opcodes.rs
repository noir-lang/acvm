use crate::{
    memory::ArrayIndex,
    value::{Typ, Value},
    RegisterIndex,
};
use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterMemIndex {
    Register(RegisterIndex),
    Constant(FieldElement),
    Memory(ArrayIndex),
}

impl RegisterMemIndex {
    pub fn to_register_index(self) -> Option<RegisterIndex> {
        match self {
            RegisterMemIndex::Register(register) => Some(register),
            RegisterMemIndex::Constant(_) | RegisterMemIndex::Memory(_) => None,
        }
    }
}

pub type Label = usize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    /// Takes the values in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryOp {
        result_type: Typ,
        op: BinaryOp,
        lhs: RegisterMemIndex,
        rhs: RegisterMemIndex,
        result: RegisterIndex,
    },
    JMPIFNOT {
        condition: RegisterMemIndex,
        destination: Label,
    },
    /// Sets the program counter to the value located at `destination`
    /// If the value at condition is non-zero
    JMPIF {
        condition: RegisterMemIndex,
        destination: Label,
    },
    /// Sets the program counter to the label.
    JMP {
        destination: Label,
    },
    // TODO:This is used to call functions and setup things like
    // TODO execution contexts.
    Call,
    // TODO:These are special functions like sha256
    Intrinsics,
    // TODO:This will be used to get data from an outside source
    Oracle {
        inputs: Vec<RegisterMemIndex>,
        destination: Vec<RegisterIndex>,
    },
    Mov {
        destination: RegisterMemIndex,
        source: RegisterMemIndex,
    },
    /// Used if execution fails during evaluation
    Trap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Cmp(Comparison),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Comparison {
    NotEqual,
    Equal,
}

impl BinaryOp {
    pub fn function(&self) -> fn(Value, Value) -> Value {
        match self {
            BinaryOp::Add => |a: Value, b: Value| a + b,
            BinaryOp::Sub => |a: Value, b: Value| a - b,
            BinaryOp::Mul => |a: Value, b: Value| a * b,
            BinaryOp::Div => |a: Value, b: Value| a / b,
            // TODO: only support equal and not equal, need less than, greater than, etc.
            BinaryOp::Cmp(_) => |a: Value, b: Value| (a == b).into(),
        }
    }
}
