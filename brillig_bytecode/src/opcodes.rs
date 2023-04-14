use std::ops::{Add, Mul, Sub};

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
    pub inputs: Vec<RegisterMemIndex>,
    /// Input values
    pub input_values: Vec<FieldElement>,
    /// Output register
    pub output: RegisterIndex,
    /// Output values - they are computed by the (external) oracle once the inputs are known
    pub output_values: Vec<FieldElement>,
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
    // pub fn function(&self) -> fn(Value, Value) -> Value {
    //     match self {
    //         BinaryOp::Add => |a: Value, b: Value| a + b,
    //         BinaryOp::Sub => |a: Value, b: Value| a - b,
    //         BinaryOp::Mul => |a: Value, b: Value| a * b,
    //         BinaryOp::Div => |a: Value, b: Value| a / b,
    //         BinaryOp::Cmp(comparison) => match comparison {
    //             Comparison::Eq => |a: Value, b: Value| (a == b).into(),
    //             Comparison::Lt => |a: Value, b: Value| (a.inner < b.inner).into(),
    //             Comparison::Lte => |a: Value, b: Value| (a.inner <= b.inner).into(),
    //         },
    //     }
    // }

    pub fn evaluate(&self, a: Value, b: Value, res_type: Typ) -> Value {
        match self {
            BinaryOp::Add => {
                let res_inner = self.wrapping(a.inner, b.inner, res_type, u128::add, Add::add);
                Value { typ: res_type, inner: res_inner }
            }
            BinaryOp::Sub => {
                let res_inner =
                    self.wrapping(a.inner, b.inner, res_type, u128::wrapping_sub, Sub::sub);
                Value { typ: res_type, inner: res_inner }
            }
            BinaryOp::Mul => {
                let res_inner = self.wrapping(a.inner, b.inner, res_type, u128::mul, Mul::mul);
                Value { typ: res_type, inner: res_inner }
            }
            BinaryOp::Div => match res_type {
                Typ::Field => a / b,
                Typ::Unsigned { bit_size } => {
                    let lhs = a.inner.to_u128() % (1_u128 << bit_size);
                    let rhs = b.inner.to_u128() % (1_u128 << bit_size);
                    Value { typ: res_type, inner: FieldElement::from(lhs / rhs) }
                }
                Typ::Signed { bit_size } => {
                    let a = field_to_signed(a.inner, bit_size);
                    let b = field_to_signed(b.inner, bit_size);
                    let res_inner = signed_to_field(a / b, bit_size);
                    Value { typ: res_type, inner: res_inner }
                }
            },
            BinaryOp::Cmp(comparison) => match comparison {
                Comparison::Eq => (a == b).into(),
                Comparison::Lt => (a.inner < b.inner).into(),
                Comparison::Lte => (a.inner <= b.inner).into(),
            },
        }
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
