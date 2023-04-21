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
    PushStack {
        source: RegisterMemIndex,
    },
    // TODO:This is used to call functions and setup things like
    // TODO execution contexts.
    CallBack,
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
            Opcode::PushStack { .. } => "pushstack",
            Opcode::CallBack => "callback",
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
    pub outputs: Vec<OracleOutput>,
    /// Output values - they are computed by the (external) oracle once the inputs are known
    pub output_values: Vec<FieldElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleInput {
    RegisterMemIndex(RegisterMemIndex),
    Array { start: RegisterMemIndex, length: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleOutput {
    RegisterIndex(RegisterIndex),
    Array { start: RegisterMemIndex, length: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
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
    pub fn evaluate(&self, a: Value, b: Value, res_type: Typ) -> Value {
        let res_inner = match self {
            BinaryOp::Add => self.wrapping(a.inner, b.inner, res_type, u128::add, Add::add),
            BinaryOp::Sub => {
                self.wrapping(a.inner, b.inner, res_type, u128::wrapping_sub, Sub::sub)
            }
            BinaryOp::Mul => self.wrapping(a.inner, b.inner, res_type, u128::mul, Mul::mul),
            BinaryOp::Div => match res_type {
                Typ::Field => a.inner / b.inner,
                Typ::Unsigned { bit_size } => {
                    let lhs = a.inner.to_u128() % (1_u128 << bit_size);
                    let rhs = b.inner.to_u128() % (1_u128 << bit_size);
                    FieldElement::from(lhs / rhs)
                }
                Typ::Signed { bit_size } => {
                    let a = field_to_signed(a.inner, bit_size);
                    let b = field_to_signed(b.inner, bit_size);
                    signed_to_field(a / b, bit_size)
                }
            },
            BinaryOp::Cmp(comparison) => match comparison {
                Comparison::Eq => ((a == b) as u128).into(),
                Comparison::Lt => ((a.inner < b.inner) as u128).into(),
                Comparison::Lte => ((a.inner <= b.inner) as u128).into(),
            },
            BinaryOp::And => {
                self.wrapping(a.inner, b.inner, res_type, u128::bitand, field_op_not_allowed)
            }
            BinaryOp::Or => {
                self.wrapping(a.inner, b.inner, res_type, u128::bitor, field_op_not_allowed)
            }
            BinaryOp::Xor => {
                self.wrapping(a.inner, b.inner, res_type, u128::bitxor, field_op_not_allowed)
            }
            BinaryOp::Shl => {
                self.wrapping(a.inner, b.inner, res_type, u128::shl, field_op_not_allowed)
            }
            BinaryOp::Shr => {
                self.wrapping(a.inner, b.inner, res_type, u128::shr, field_op_not_allowed)
            }
        };

        Value { typ: res_type, inner: res_inner }
    }

    /// Perform the given numeric operation and modulo the result by the max value for the given bit count
    /// if the res_type is not a FieldElement.
    fn wrapping(
        &self,
        lhs: FieldElement,
        rhs: FieldElement,
        res_type: Typ,
        u128_op: impl FnOnce(u128, u128) -> u128,
        field_op: impl FnOnce(FieldElement, FieldElement) -> FieldElement,
    ) -> FieldElement {
        match res_type {
            Typ::Field => field_op(lhs, rhs),
            Typ::Unsigned { bit_size } | Typ::Signed { bit_size } => {
                let type_modulo = 1_u128 << bit_size;
                let lhs = lhs.to_u128() % type_modulo;
                let rhs = rhs.to_u128() % type_modulo;
                let mut x = u128_op(lhs, rhs);
                x %= type_modulo;
                FieldElement::from(x)
            }
        }
    }
}

fn field_to_signed(f: FieldElement, n: u32) -> i128 {
    assert!(n < 127);
    let a = f.to_u128();
    let pow_2 = 2_u128.pow(n);
    if a < pow_2 {
        a as i128
    } else {
        (a - 2 * pow_2) as i128
    }
}

fn signed_to_field(a: i128, n: u32) -> FieldElement {
    if n >= 126 {
        panic!("ICE: cannot convert signed {n} bit size into field");
    }
    if a >= 0 {
        FieldElement::from(a)
    } else {
        let b = (a + 2_i128.pow(n + 1)) as u128;
        FieldElement::from(b)
    }
}

fn field_op_not_allowed(_lhs: FieldElement, _rhs: FieldElement) -> FieldElement {
    unreachable!("operation not allowed for FieldElement");
}
