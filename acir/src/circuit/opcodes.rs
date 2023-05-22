use super::directives::{Directive, LogInfo, QuotientDirective};
use crate::native_types::{Expression, Witness};

use brillig_bytecode::ForeignCallResult;
use serde::{Deserialize, Serialize};

mod black_box_function_call;
mod block;
mod oracle_data;

pub use black_box_function_call::{BlackBoxFuncCall, FunctionInput};
pub use block::{BlockId, MemOp, MemoryBlock};
pub use oracle_data::OracleData;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    Arithmetic(Expression),
    BlackBoxFuncCall(BlackBoxFuncCall),
    Directive(Directive),
    /// Abstract read/write operations on a block of data. In particular;
    /// It does not require an initialisation phase
    /// Operations do not need to be constant, they can be any expression which resolves to 0 or 1.
    Block(MemoryBlock),
    /// Same as Block, but it starts with an initialisation phase and then have only read operation
    /// - init: write operations with index from 0..MemoryBlock.len
    /// - after MemoryBlock.len; all operations are read
    /// ROM can be more efficiently handled because we do not need to check for the operation value (which is always 0).
    ROM(MemoryBlock),
    /// Same as ROM, but can have read or write operations
    /// - init = write operations with index 0..MemoryBlock.len
    /// - after MemoryBlock.len, all operations are constant expressions (0 or 1)
    /// RAM is required for Aztec Backend as dynamic memory implementation in Barrentenberg requires an intialisation phase and can only handle constant values for operations.
    RAM(MemoryBlock),
    Oracle(OracleData),
    Brillig(Brillig),
}

/// Inputs for the Brillig VM. These are the initial inputs
/// that the Brillig VM will use to start.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BrilligInputs {
    Single(Expression),
    Array(Vec<Expression>),
}

/// Outputs for the Brillig VM. Once the VM has completed
/// execution, this will the object that is returned.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BrilligOutputs {
    Simple(Witness),
    Array(Vec<Witness>),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Brillig {
    pub inputs: Vec<BrilligInputs>,
    pub outputs: Vec<BrilligOutputs>,
    /// Results of oracles/functions external to brillig like a database read
    pub foreign_call_results: Vec<ForeignCallResult>,
    pub bytecode: Vec<brillig_bytecode::Opcode>,
    /// Predicate of the Brillig execution - indicates if it should be skipped
    pub predicate: Option<Expression>,
}

impl Opcode {
    // TODO We can add a domain separator by doing something like:
    // TODO concat!("directive:", directive.name)
    pub fn name(&self) -> &str {
        match self {
            Opcode::Arithmetic(_) => "arithmetic",
            Opcode::Directive(directive) => directive.name(),
            Opcode::BlackBoxFuncCall(g) => g.name(),
            Opcode::Block(_) => "block",
            Opcode::RAM(_) => "ram",
            Opcode::ROM(_) => "rom",
            Opcode::Oracle(data) => &data.name,
            Opcode::Brillig(_) => "brillig",
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
                    write!(f, "({}, _{}, _{}) ", i.0, i.1.witness_index(), i.2.witness_index())?;
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
            Opcode::Directive(Directive::Quotient(QuotientDirective { a, b, q, r, predicate })) => {
                write!(f, "DIR::QUOTIENT ")?;
                if let Some(pred) = predicate {
                    writeln!(f, "PREDICATE = {pred}")?;
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
            Opcode::BlackBoxFuncCall(g) => write!(f, "{g}"),
            Opcode::Directive(Directive::ToLeRadix { a, b, radix: _ }) => {
                write!(f, "DIR::TORADIX ")?;
                write!(
                    f,
                    // TODO (Note): this assumes that the decomposed bits have contiguous witness indices
                    // This should be the case, however, we can also have a function which checks this
                    "(_{}, [_{}..._{}] )",
                    a,
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
            Opcode::Directive(Directive::PermutationSort { inputs: a, tuple, bits, sort_by }) => {
                write!(f, "DIR::PERMUTATIONSORT ")?;
                write!(
                    f,
                    "(permutation size: {} {}-tuples, sort_by: {:#?}, bits: [_{}..._{}]))",
                    a.len(),
                    tuple,
                    sort_by,
                    // (Note): the bits do not have contiguous index but there are too many for display
                    bits.first().unwrap().witness_index(),
                    bits.last().unwrap().witness_index(),
                )
            }
            Opcode::Directive(Directive::Log(info)) => match info {
                LogInfo::FinalizedOutput(output_string) => write!(f, "Log: {output_string}"),
                LogInfo::WitnessOutput(witnesses) => write!(
                    f,
                    "Log: _{}..._{}",
                    witnesses.first().unwrap().witness_index(),
                    witnesses.last().unwrap().witness_index()
                ),
            },
            Opcode::Block(block) => {
                write!(f, "BLOCK ")?;
                write!(f, "(id: {}, len: {}) ", block.id.0, block.trace.len())
            }
            Opcode::ROM(block) => {
                write!(f, "ROM ")?;
                write!(f, "(id: {}, len: {}) ", block.id.0, block.trace.len())
            }
            Opcode::RAM(block) => {
                write!(f, "RAM ")?;
                write!(f, "(id: {}, len: {}) ", block.id.0, block.trace.len())
            }
            Opcode::Oracle(data) => {
                write!(f, "ORACLE: ")?;
                write!(f, "{data}")
            }
            Opcode::Brillig(brillig) => {
                write!(f, "BRILLIG: ")?;
                writeln!(f, "inputs: {:?}", brillig.inputs)?;
                writeln!(f, "outputs: {:?}", brillig.outputs)?;
                writeln!(f, "{:?}", brillig.bytecode)
            }
        }
    }
}

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
