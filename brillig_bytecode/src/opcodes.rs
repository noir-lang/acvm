use crate::{
    RegisterIndex,
};
use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

pub type Label = usize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    /// Takes the fields in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryFieldOp {
        destination: RegisterIndex,
        op: BinaryFieldOp,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
    },
    /// Takes the bit_size size integers in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryIntOp {
        destination: RegisterIndex,
        op: BinaryIntOp,
        bit_size: u32,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
    },
    JumpIfNot {
        condition: RegisterIndex,
        location: Label,
    },
    /// Sets the program counter to the value located at `destination`
    /// If the value at condition is non-zero
    JumpIf {
        condition: RegisterIndex,
        location: Label,
    },
    /// Sets the program counter to the label.
    Jump {
        location: Label,
    },
    // We don't support dynamic jumps or calls
    // See https://github.com/ethereum/aleth/issues/3404 for reasoning
    Call {
        location: Label,
    },
    Const {
        destination: RegisterIndex,
        value: FieldElement,  
    },
    Return,
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
        source_pointer: RegisterIndex,
    },
    Store {
        destination_pointer: RegisterIndex,
        source: RegisterIndex,
    },
    /// Used if execution fails during evaluation
    Trap,
    /// Stop execution
    Stop,
}

impl Opcode {
    pub fn name(&self) -> &'static str {
        match self {
            Opcode::BinaryFieldOp { .. } => "binary_field_op",
            Opcode::BinaryIntOp { .. } => "binary_int_op",
            Opcode::JumpIfNot { .. } => "jmp_if_not",
            Opcode::JumpIf { .. } => "jmp_if",
            Opcode::Jump { .. } => "jmp",
            Opcode::Call { .. } => "call",
            Opcode::Const { .. } => "const",
            Opcode::Return => "return",
            Opcode::Intrinsics => "intrinsics",
            Opcode::Oracle(_) => "oracle",
            Opcode::Mov { .. } => "mov",
            Opcode::Load { .. } => "load",
            Opcode::Store { .. } => "store",
            Opcode::Trap => "trap",
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
    /// Output registers
    pub outputs: Vec<OracleOutput>,
    /// Output values - they are computed by the (external) oracle once the inputs are known
    pub output_values: Vec<FieldElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleInput {
    RegisterIndex(RegisterIndex),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleOutput {
    RegisterIndex(RegisterIndex),
}


// Binary fixed-length integer expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryFieldOp {
    Add,
    Sub,
    Mul,
    Div,
    Cmp(Comparison),
}

// Binary fixed-length integer expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryIntOp {
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

impl BinaryFieldOp {
    /// Evaluate a binary operation on two FieldElements and return the result as a FieldElement.
    pub fn evaluate_field(&self, a: FieldElement, b: FieldElement) -> FieldElement {
        match self {
            // Perform addition, subtraction, multiplication, and division based on the BinaryOp variant.
            BinaryFieldOp::Add => a + b,
            BinaryFieldOp::Sub => a - b,
            BinaryFieldOp::Mul => a * b,
            BinaryFieldOp::Div => a / b,
             // Perform a comparison between a and b based on the Comparison variant.
             BinaryFieldOp::Cmp(comparison) => match comparison {
                Comparison::Eq => ((a == b) as u128).into(),
                Comparison::Lt => ((a < b) as u128).into(),
                Comparison::Lte => ((a <= b) as u128).into(),
            },
        }
    }
}

impl BinaryIntOp {
    /// Evaluate a binary operation on two unsigned integers (u128) with a given bit size and return the result as a u128.
    pub fn evaluate_int(&self, a: u128, b: u128, bit_size: u32) -> u128 {
        let bit_modulo = 1_u128 << bit_size;
        match self {
            // Perform addition, subtraction, and multiplication, applying a modulo operation to keep the result within the bit size.
            BinaryIntOp::Add => (a + b) % bit_modulo,
            BinaryIntOp::Sub => (a - b) % bit_modulo,
            BinaryIntOp::Mul => (a * b) % bit_modulo,
            // Perform unsigned division using the modulo operation on a and b.
            BinaryIntOp::UnsignedDiv => (a % bit_modulo) / (b % bit_modulo),
            // Perform signed division by first converting a and b to signed integers and then back to unsigned after the operation.
            BinaryIntOp::SignedDiv => to_unsigned(to_signed(a, bit_size) / to_signed(b, bit_size), bit_size),
            // Perform a comparison between a and b based on the Comparison variant.
            BinaryIntOp::Cmp(comparison) => match comparison {
                Comparison::Eq => (a == b) as u128,
                Comparison::Lt => (a < b) as u128,
                Comparison::Lte => (a <= b) as u128,
            },
            // Perform bitwise AND, OR, XOR, left shift, and right shift operations, applying a modulo operation to keep the result within the bit size.
            BinaryIntOp::And => {
                (a & b) % bit_modulo
            }
            BinaryIntOp::Or => {
                (a | b) % bit_modulo
            }
            BinaryIntOp::Xor => {
                (a ^ b) % bit_modulo
            }
            BinaryIntOp::Shl => {
                (a << b) % bit_modulo
            }
            BinaryIntOp::Shr => {
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
        // TODO(AD): clean this up a bit - this is only converted to a field later, error there?
        panic!("ICE: cannot convert signed {n} bit size into field");
    }
    if a >= 0 {
        a as u128
    } else {
        (a + 2_i128.pow(n + 1)) as u128
    }
}