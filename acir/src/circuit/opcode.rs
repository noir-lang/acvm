use serde::{Deserialize, Serialize};

use crate::native_types::{Expression, Witness};
use crate::BlackBoxFunc;

use super::directives::Directive;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
// XXX: Gate does not capture what this is anymore. I think IR/OPCODE would be a better name
pub enum Opcode {
    Arithmetic(Expression),
    BlackBoxFuncCall(BlackBoxFuncCall),
    Directive(Directive),
}

impl Opcode {
    // TODO: This will be deprecated when we flatten the IR
    pub fn name(&self) -> &str {
        match self {
            Opcode::Arithmetic(_) => "arithmetic",
            Opcode::Directive(directive) => directive.name(),
            Opcode::BlackBoxFuncCall(g) => g.name.name(),
        }
    }
    // We have three types of opcodes allowed in the IR
    // Expression, BlackBoxFuncCall and Directives
    // When we serialise these opcodes, we use the index
    // to uniquely identify which category of opcode we are dealing with.
    pub fn to_index(&self) -> u8 {
        match self {
            Opcode::Arithmetic(_) => 0,
            Opcode::BlackBoxFuncCall(_) => 1,
            Opcode::Directive(_) => 2,
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(self, Opcode::Arithmetic(_))
    }
    pub fn arithmetic(self) -> Option<Expression> {
        match self {
            Opcode::Arithmetic(gate) => Some(gate),
            _ => None,
        }
    }
}

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Arithmetic(a) => {
                for i in &a.mul_terms {
                    write!(
                        f,
                        "{:?}x{}*x{} + ",
                        i.0,
                        i.1.witness_index(),
                        i.2.witness_index()
                    )?;
                }
                for i in &a.linear_combinations {
                    write!(f, "{:?}x{} + ", i.0, i.1.witness_index())?;
                }
                write!(f, "{:?} = 0", a.q_c)
            }
            Opcode::Directive(Directive::Invert { x, result: r }) => {
                write!(f, "x{}=1/x{}, or 0", r.witness_index(), x.witness_index())
            }
            Opcode::Directive(Directive::Truncate {
                a,
                b,
                c: _c,
                bit_size,
            }) => {
                write!(
                    f,
                    "Truncate: x{} is x{} truncated to {} bits",
                    b.witness_index(),
                    a.witness_index(),
                    bit_size
                )
            }
            Opcode::Directive(Directive::Quotient {
                a,
                b,
                q,
                r,
                predicate,
            }) => {
                if let Some(pred) = predicate {
                    write!(
                        f,
                        "Predicate euclidian division: {}*{} = {}*(x{}*{} + x{})",
                        pred,
                        a,
                        pred,
                        q.witness_index(),
                        b,
                        r.witness_index()
                    )
                } else {
                    write!(
                        f,
                        "Euclidian division: {} = x{}*{} + x{}",
                        a,
                        q.witness_index(),
                        b,
                        r.witness_index()
                    )
                }
            }
            Opcode::Directive(Directive::Oddrange { a, b, r, bit_size }) => {
                write!(
                    f,
                    "Oddrange: x{} = x{}*2^{} + x{}",
                    a.witness_index(),
                    b.witness_index(),
                    bit_size,
                    r.witness_index()
                )
            }
            Opcode::BlackBoxFuncCall(g) => write!(f, "{:?}", g),
            Opcode::Directive(Directive::Split { a, b, bit_size: _ }) => {
                write!(
                    f,
                    "Split: {} into x{}...x{}",
                    a,
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
            Opcode::Directive(Directive::ToBytes { a, b, byte_size: _ }) => {
                write!(
                    f,
                    "To Bytes: {} into x{}...x{}",
                    a,
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
        }
    }
}

// Note: Some functions will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionInput {
    pub witness: Witness,
    pub num_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackBoxFuncCall {
    pub name: BlackBoxFunc,
    pub inputs: Vec<FunctionInput>,
    pub outputs: Vec<Witness>,
}
