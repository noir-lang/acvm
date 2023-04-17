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
    Call {
        destination: RegisterMemIndex,
    },
    // TODO:These are special functions like sha256
    Intrinsics,
    /// Used to get data from an outside source
    Oracle(OracleData),
    Mov {
        destination: RegisterMemIndex,
        source: RegisterMemIndex,
    },
    Load {
        destination: RegisterMemIndex,
        array_id_reg: RegisterMemIndex,
        index: RegisterMemIndex,
    },
    Store {
        source: RegisterMemIndex,
        array_id_reg: RegisterMemIndex,
        index: RegisterMemIndex,
    },
    /// Used if execution fails during evaluation
    Trap,
    /// Hack
    Bootstrap {
        register_allocation_indices: Vec<u32>,
    },
    /// Stop execution
    Stop,
}

impl Opcode {
    pub fn name(&self) -> &'static str {
        match self {
            Opcode::BinaryOp { .. } => "binary_op",
            Opcode::JMPIFNOT { .. } => "jmpifnot",
            Opcode::JMPIF { .. } => "jmpif",
            Opcode::JMP { .. } => "jmp",
            Opcode::Call { .. } => "call",
            Opcode::Intrinsics => "intrinsics",
            Opcode::Oracle(_) => "oracle",
            Opcode::Mov { .. } => "mov",
            Opcode::Load { .. } => "load",
            Opcode::Store { .. } => "store",
            Opcode::Trap => "trap",
            Opcode::Bootstrap { .. } => "bootstrap",
            Opcode::Stop => "stop",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleData {
    /// Name of the oracle
    pub name: String,
    /// Input registers
    pub inputs: Vec<OracleInput>,
    /// Input values
    pub input_values: Vec<FieldElement>,
    /// Output register
    pub output: RegisterIndex,
    /// Output values - they are computed by the (external) oracle once the inputs are known
    pub output_values: Vec<FieldElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleInput {
    pub register_mem_index: RegisterMemIndex,
    pub length: usize,
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
    Eq,  //(==) equal
    Lt,  //(<) field less
    Lte, //(<=) field less or equal
}

impl BinaryOp {
    pub fn function(&self) -> fn(Value, Value) -> Value {
        match self {
            BinaryOp::Add => |a: Value, b: Value| a + b,
            BinaryOp::Sub => |a: Value, b: Value| a - b,
            BinaryOp::Mul => |a: Value, b: Value| a * b,
            BinaryOp::Div => |a: Value, b: Value| a / b,
            BinaryOp::Cmp(comparison) => match comparison {
                Comparison::Eq => |a: Value, b: Value| (a == b).into(),
                Comparison::Lt => |a: Value, b: Value| (a.inner < b.inner).into(),
                Comparison::Lte => |a: Value, b: Value| (a.inner <= b.inner).into(),
            },
        }
    }
}
