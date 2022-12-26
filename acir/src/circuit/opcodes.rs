use super::directives::Directive;
use crate::native_types::{Expression, Witness};
use crate::BlackBoxFunc;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    Arithmetic(Expression),
    BlackBoxFuncCall(BlackBoxFuncCall),
    Directive(Directive),
}

impl Opcode {
    // TODO We can add a domain separator by doing something like:
    // TODO concat!("directive:", directive.name)
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
            Opcode::Arithmetic(expr) => Some(expr),
            _ => None,
        }
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Arithmetic(expr) => {
                write!(f, "EXPR [ ")?;
                for i in &expr.mul_terms {
                    write!(
                        f,
                        "({}, _{}, _{}) ",
                        i.0,
                        i.1.witness_index(),
                        i.2.witness_index()
                    )?;
                }
                for i in &expr.linear_combinations {
                    write!(f, "({}, _{}) ", i.0, i.1.witness_index())?;
                }
                write!(f, "{}", expr.q_c)?;

                write!(f, " ]")
            }
            Opcode::Directive(Directive::Invert { x, result: r }) => {
                write!(f, "DIR::INVERT ")?;
                write!(f, "(_{}, out: _{}) ", x.witness_index(), r.witness_index())
            }
            Opcode::Directive(Directive::Truncate { a, b, c, bit_size }) => {
                write!(f, "DIR::TRUNCATE ")?;
                write!(
                    f,
                    "(out: _{}, _{}, _{}, bit_size: {})",
                    // TODO: Modify Noir to switch a and b
                    b.witness_index(),
                    a.witness_index(),
                    // TODO: check why c was unused before, and check when directive is being processed
                    // TODO: and if it is used
                    c.witness_index(),
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
                write!(f, "DIR::QUOTIENT ")?;
                if let Some(pred) = predicate {
                    writeln!(f, "PREDICATE = {}", pred)?;
                }

                write!(
                    f,
                    "(out : _{},  (_{}, {}), _{})",
                    a,
                    q.witness_index(),
                    b,
                    r.witness_index()
                )
            }
            Opcode::Directive(Directive::Oddrange { a, b, r, bit_size }) => {
                write!(f, "DIR::ODDRANGE ")?;

                write!(
                    f,
                    "(out: _{}, (_{}, bit_size: {}), _{})",
                    a.witness_index(),
                    b.witness_index(),
                    bit_size,
                    r.witness_index()
                )
            }
            Opcode::BlackBoxFuncCall(g) => write!(f, "{}", g),
            Opcode::Directive(Directive::ToBits { a, b, bit_size: _ }) => {
                write!(f, "DIR::TOBITS ")?;
                write!(
                    f,
                    // TODO (Note): this assumes that the decomposed bits have contiguous witness indices
                    // This should be the case, however, we can also have a function which checks this
                    "(_{}, [_{}..._{}])",
                    a,
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
            Opcode::Directive(Directive::ToBytes { a, b, byte_size: _ }) => {
                write!(f, "DIR::TOBYTES ")?;
                write!(
                    f,
                    "(_{}, [_{}..._{}])",
                    a,
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
        }
    }
}

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

// Note: Some functions will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionInput {
    pub witness: Witness,
    pub num_bits: u32,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackBoxFuncCall {
    pub name: BlackBoxFunc,
    pub inputs: Vec<FunctionInput>,
    pub outputs: Vec<Witness>,
}

impl std::fmt::Display for BlackBoxFuncCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uppercase_name: String = self.name.name().into();
        let uppercase_name = uppercase_name.to_uppercase();
        write!(f, "BLACKBOX::{} ", uppercase_name)?;
        write!(f, "[")?;

        // Once a vectors length gets above this limit,
        // instead of listing all of their elements, we use ellipses
        // t abbreviate them
        const ABBREVIATION_LIMIT: usize = 5;

        let should_abbreviate_inputs = self.inputs.len() <= ABBREVIATION_LIMIT;
        let should_abbreviate_outputs = self.outputs.len() <= ABBREVIATION_LIMIT;

        // INPUTS
        //
        let inputs_str = if should_abbreviate_inputs {
            let mut result = String::new();
            for (index, inp) in self.inputs.iter().enumerate() {
                result += &format!(
                    "(_{}, num_bits: {})",
                    inp.witness.witness_index(),
                    inp.num_bits
                );
                // Add a comma, unless it is the last entry
                if index != self.inputs.len() - 1 {
                    result += ", "
                }
            }
            result
        } else {
            let first = self.inputs.first().unwrap();
            let last = self.inputs.last().unwrap();

            let mut result = String::new();

            result += &format!(
                "(_{}, num_bits: {})...(_{}, num_bits: {})",
                first.witness.witness_index(),
                first.num_bits,
                last.witness.witness_index(),
                last.num_bits,
            );

            result
        };
        write!(f, "{}", inputs_str)?;
        write!(f, "] ")?;

        // OUTPUTS
        // TODO: Avoid duplication of INPUTS and OUTPUTS code

        if self.outputs.is_empty() {
            return Ok(());
        }

        write!(f, "[ ")?;
        let outputs_str = if should_abbreviate_outputs {
            let mut result = String::new();
            for (index, output) in self.outputs.iter().enumerate() {
                result += &format!("_{}", output.witness_index());
                // Add a comma, unless it is the last entry
                if index != self.outputs.len() - 1 {
                    result += ", "
                }
            }
            result
        } else {
            let first = self.outputs.first().unwrap();
            let last = self.outputs.last().unwrap();

            let mut result = String::new();
            result += &format!("(_{},...,_{})", first.witness_index(), last.witness_index());
            result
        };
        write!(f, "{}", outputs_str)?;
        write!(f, "]")
    }
}

impl std::fmt::Debug for BlackBoxFuncCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
