use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Shl, Shr, Sub};

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
    /// Takes the fields in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryFieldOp {
        op: BinaryOp,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
    },
    /// Takes the bit_size size integers in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryIntOp {
        op: BinaryOp,
        bit_size: u32,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
    },
    JumpIfNot {
        condition: RegisterIndex,
        destination: Label,
    },
    /// Sets the program counter to the value located at `destination`
    /// If the value at condition is non-zero
    JumpIf {
        condition: RegisterIndex,
        destination: Label,
    },
    /// Sets the program counter to the label.
    Jump {
        destination: Label,
    },
    PushStack {
        source: RegisterIndex,
    },
    // TODO:This is used to call functions and setup things like
    // TODO execution contexts.
    Call,
    // TODO:These are special functions like sha256
    Intrinsics,
    /// Used to get data from an outside source
    Oracle(OracleData),
    Mov {
        destination: RegisterIndex,
        source: RegisterIndex,
    },
    Load {
        destination: RegisterIndex,
        array_id_reg: RegisterIndex,
        index: RegisterIndex,
    },
    LoadConst {
        destination: RegisterIndex,
        constant: FieldElement,
    },
    Store {
        source: RegisterIndex,
        array_id_reg: RegisterIndex,
        index: RegisterIndex,
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
            Opcode::BinaryFieldOp { .. } => "binary_field_op",
            Opcode::BinaryIntOp { .. } => "binary_int_op",
            Opcode::JumpIfNot { .. } => "jmpifnot",
            Opcode::JumpIf { .. } => "jmpif",
            Opcode::Jump { .. } => "jmp",
            Opcode::PushStack { .. } => "pushstack",
            Opcode::Call => "callback",
            Opcode::Intrinsics => "intrinsics",
            Opcode::Oracle(_) => "oracle",
            Opcode::Mov { .. } => "mov",
            Opcode::Load { .. } => "load",
            Opcode::Store { .. } => "store",
            Opcode::Trap => "trap",
            Opcode::Bootstrap { .. } => "bootstrap",
            Opcode::Stop => "stop",
            Opcode::LoadConst { .. } => "loadconst",
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
    /// Output registers
    pub outputs: Vec<OracleOutput>,
    /// Output values - they are computed by the (external) oracle once the inputs are known
    pub output_values: Vec<FieldElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleInput {
    RegisterIndex(RegisterIndex),
    Array { start: RegisterIndex, length: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleOutput {
    RegisterIndex(RegisterIndex),
    Array { start: RegisterIndex, length: usize },
}


// Binary fixed-length integer expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    SignedDiv,
    UnsignedDiv,
    Cmp(Comparison),
    And, // (&) Bitwise AND
    Or,  // (|) Bitwise OR
    Xor, // (^) Bitwise XOR
    Shl, // (<<) Shift left
    Shr, // (>>) Shift right
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Comparison {
    Eq,  //(==) equal
    Lt,  //(<) field less
    Lte, //(<=) field less or equal
}

impl BinaryOp {
    /// Evaluate a binary operation on two FieldElements and return the result as a FieldElement.
    pub fn evaluate_field(&self, a: FieldElement, b: FieldElement) -> FieldElement {
        match self {
            // Perform addition, subtraction, multiplication, and division based on the BinaryOp variant.
            BinaryOp::Add => a + b,
            BinaryOp::Sub => a - b,
            BinaryOp::Mul => a * b,
            BinaryOp::SignedDiv | BinaryOp::UnsignedDiv => a / b,
             // Perform a comparison between a and b based on the Comparison variant.
             BinaryOp::Cmp(comparison) => match comparison {
                Comparison::Eq => ((a == b) as u128).into(),
                Comparison::Lt => ((a < b) as u128).into(),
                Comparison::Lte => ((a <= b) as u128).into(),
            },
            // These operations are not allowed for FieldElement, so they are unreachable.
            BinaryOp::And => unreachable!("operation not allowed for FieldElement"),
            BinaryOp::Or => unreachable!("operation not allowed for FieldElement"),
            BinaryOp::Xor => unreachable!("operation not allowed for FieldElement"),
            BinaryOp::Shl => unreachable!("operation not allowed for FieldElement"),
            BinaryOp::Shr => unreachable!("operation not allowed for FieldElement"),
        }
    }
    /// Evaluate a binary operation on two unsigned integers (u128) with a given bit size and return the result as a u128.
    pub fn evaluate_int(&self, a: u128, b: u128, bit_size: u32) -> u128 {
        let bit_modulo = 1_u128 << bit_size;
        match self {
            // Perform addition, subtraction, and multiplication, applying a modulo operation to keep the result within the bit size.
            BinaryOp::Add => (a + b) % bit_modulo,
            BinaryOp::Sub => (a - b) % bit_modulo,
            BinaryOp::Mul => (a * b) % bit_modulo,
            // Perform unsigned division using the modulo operation on a and b.
            BinaryOp::UnsignedDiv => (a % bit_modulo) / (b % bit_modulo),
            // Perform signed division by first converting a and b to signed integers and then back to unsigned after the operation.
            BinaryOp::SignedDiv => to_unsigned(to_signed(a, bit_size) / to_signed(b, bit_size), bit_size),
            // Perform a comparison between a and b based on the Comparison variant.
            BinaryOp::Cmp(comparison) => match comparison {
                Comparison::Eq => ((a == b) as u128).into(),
                Comparison::Lt => ((a < b) as u128).into(),
                Comparison::Lte => ((a <= b) as u128).into(),
            },
            // Perform bitwise AND, OR, XOR, left shift, and right shift operations, applying a modulo operation to keep the result within the bit size.
            BinaryOp::And => {
                (a & b) % bit_modulo
            }
            BinaryOp::Or => {
                (a | b) % bit_modulo
            }
            BinaryOp::Xor => {
                (a ^ b) % bit_modulo
            }
            BinaryOp::Shl => {
                (a << b) % bit_modulo
            }
            BinaryOp::Shr => {
                (a >> b) % bit_modulo
            }
        }
    }
}

fn to_signed(a: u128, n: u32) -> i128 {
    assert!(n < 127);
    let pow_2 = 2_u128.pow(n);
    if a < pow_2 {
        a as i128
    } else {
        (a - 2 * pow_2) as i128
    }
}

fn to_unsigned(a: i128, n: u32) -> u128 {
    if n >= 126 {
        panic!("ICE: cannot convert signed {n} bit size into field");
    }
    if a >= 0 {
        a as u128
    } else {
        (a + 2_i128.pow(n + 1)) as u128
    }
}