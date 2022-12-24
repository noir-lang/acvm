use serde::{Deserialize, Serialize};

use crate::native_types::{Expression, Witness};
use crate::OPCODE;

use super::directives::Directive;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
// XXX: Gate does not capture what this is anymore. I think IR/OPCODE would be a better name
pub enum Gate {
    Arithmetic(Expression),
    GadgetCall(GadgetCall),
    Directive(Directive),
}

impl Gate {
    // TODO: This will be deprecated when we flatten the IR
    pub fn name(&self) -> &str {
        match self {
            Gate::Arithmetic(_) => "arithmetic",
            Gate::Directive(Directive::Invert { .. }) => "invert",
            Gate::Directive(Directive::Truncate { .. }) => "truncate",
            Gate::Directive(Directive::Quotient { .. }) => "quotient",
            Gate::Directive(Directive::Oddrange { .. }) => "odd_range",
            Gate::Directive(Directive::Split { .. }) => "split",
            Gate::Directive(Directive::ToBytes { .. }) => "to_bytes",
            Gate::GadgetCall(g) => g.name.name(),
        }
    }
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, Gate::Arithmetic(_))
    }
    pub fn arithmetic(self) -> Expression {
        match self {
            Gate::Arithmetic(gate) => gate,
            _ => panic!("tried to convert a non arithmetic gate to an Expression struct"),
        }
    }
}

impl std::fmt::Debug for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gate::Arithmetic(a) => {
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
            Gate::Directive(Directive::Invert { x, result: r }) => {
                write!(f, "x{}=1/x{}, or 0", r.witness_index(), x.witness_index())
            }
            Gate::Directive(Directive::Truncate {
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
            Gate::Directive(Directive::Quotient {
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
            Gate::Directive(Directive::Oddrange { a, b, r, bit_size }) => {
                write!(
                    f,
                    "Oddrange: x{} = x{}*2^{} + x{}",
                    a.witness_index(),
                    b.witness_index(),
                    bit_size,
                    r.witness_index()
                )
            }
            Gate::GadgetCall(g) => write!(f, "{:?}", g),
            Gate::Directive(Directive::Split { a, b, bit_size: _ }) => {
                write!(
                    f,
                    "Split: {} into x{}...x{}",
                    a,
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
            Gate::Directive(Directive::ToBytes { a, b, byte_size: _ }) => {
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

// Note: Some gadgets will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GadgetInput {
    pub witness: Witness,
    pub num_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GadgetCall {
    pub name: OPCODE,
    pub inputs: Vec<GadgetInput>,
    pub outputs: Vec<Witness>,
}