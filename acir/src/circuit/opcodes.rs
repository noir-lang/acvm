use std::io::{Read, Write};

use super::directives::{Directive, LogInfo};
use crate::native_types::Expression;
use crate::serialization::{read_n, write_bytes};

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
}

impl Opcode {
    // TODO We can add a domain separator by doing something like:
    // TODO concat!("directive:", directive.name)
    pub fn name(&self) -> &str {
        match self {
            Opcode::Arithmetic(_) => "arithmetic",
            Opcode::Directive(directive) => directive.name(),
            Opcode::BlackBoxFuncCall(g) => g.get_black_box_func().name(),
            Opcode::Block(_) => "block",
            Opcode::RAM(_) => "ram",
            Opcode::ROM(_) => "rom",
            Opcode::Oracle(data) => &data.name,
        }
    }

    // When we serialize the opcodes, we use the index
    // to uniquely identify which category of opcode we are dealing with.
    pub(crate) fn to_index(&self) -> u8 {
        match self {
            Opcode::Arithmetic(_) => 0,
            Opcode::BlackBoxFuncCall(_) => 1,
            Opcode::Directive(_) => 2,
            Opcode::Block(_) => 3,
            Opcode::ROM(_) => 4,
            Opcode::RAM(_) => 5,
            Opcode::Oracle { .. } => 6,
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

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        let opcode_index = self.to_index();
        write_bytes(&mut writer, &[opcode_index])?;

        match self {
            Opcode::Arithmetic(expr) => expr.write(writer),
            Opcode::BlackBoxFuncCall(func_call) => func_call.write(writer),
            Opcode::Directive(directive) => directive.write(writer),
            Opcode::Block(mem_block) | Opcode::ROM(mem_block) | Opcode::RAM(mem_block) => {
                mem_block.write(writer)
            }
            Opcode::Oracle(data) => data.write(writer),
        }
    }
    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        // First byte indicates the opcode category
        let opcode_index = read_n::<1, _>(&mut reader)?[0];

        match opcode_index {
            0 => {
                let expr = Expression::read(reader)?;

                Ok(Opcode::Arithmetic(expr))
            }
            1 => {
                let func_call = BlackBoxFuncCall::read(reader)?;

                Ok(Opcode::BlackBoxFuncCall(func_call))
            }
            2 => {
                let directive = Directive::read(reader)?;
                Ok(Opcode::Directive(directive))
            }
            3 => {
                let block = MemoryBlock::read(reader)?;
                Ok(Opcode::Block(block))
            }
            4 => {
                let block = MemoryBlock::read(reader)?;
                Ok(Opcode::ROM(block))
            }
            5 => {
                let block = MemoryBlock::read(reader)?;
                Ok(Opcode::RAM(block))
            }
            6 => {
                let data = OracleData::read(reader)?;
                Ok(Opcode::Oracle(data))
            }
            _ => Err(std::io::ErrorKind::InvalidData.into()),
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
            Opcode::Directive(Directive::Quotient { a, b, q, r, predicate }) => {
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
        }
    }
}

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[test]
fn serialization_roundtrip() {
    use crate::native_types::Witness;

    fn read_write(opcode: Opcode) -> (Opcode, Opcode) {
        let mut bytes = Vec::new();
        opcode.write(&mut bytes).unwrap();
        let got_opcode = Opcode::read(&*bytes).unwrap();
        (opcode, got_opcode)
    }

    let opcode_arith = Opcode::Arithmetic(Expression::default());

    let opcode_black_box_func = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AES {
        inputs: vec![
            FunctionInput { witness: Witness(1u32), num_bits: 12 },
            FunctionInput { witness: Witness(24u32), num_bits: 32 },
        ],
        outputs: vec![Witness(123u32), Witness(245u32)],
    });

    let opcode_directive =
        Opcode::Directive(Directive::Invert { x: Witness(1234u32), result: Witness(56789u32) });

    let opcodes = vec![opcode_arith, opcode_black_box_func, opcode_directive];

    for opcode in opcodes {
        let (op, got_op) = read_write(opcode);
        assert_eq!(op, got_op)
    }
}

#[test]
fn panic_regression_187() {
    // See: https://github.com/noir-lang/acvm/issues/187
    // This issue seems to not be reproducible on Mac.
    let data = b"\x00\x00\x00\x00\xff\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x02\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x77\xdc\xa8\x37\x00\x00\x00\x06\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xf0\x80\x00\x00\x80\x00\x00\x00\x00\x00\x00\x00\x04";
    let circuit = crate::circuit::Circuit::read(&data[..]);
    assert!(circuit.is_err())
}
