use std::io::{Read, Write};

use super::directives::{Directive, LogInfo};
use crate::native_types::{Expression, Witness};
use crate::serialization::{
    read_bytes, read_field_element, read_n, read_u16, read_u32, write_bytes, write_u16, write_u32,
};
use crate::BlackBoxFunc;
use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Brillig {
    pub inputs: Vec<Expression>,
    pub outputs: Vec<Witness>,
    pub bytecode: Vec<brillig_bytecode::Opcode>,
    /// Predicate of the Brillig execution - indicates if it should be skipped
    pub predicate: Option<Expression>,
}

impl Brillig {
    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        let inputs_len = self.inputs.len() as u32;
        write_u32(&mut writer, inputs_len)?;
        for input in &self.inputs {
            input.write(&mut writer)?
        }

        let outputs_len = self.outputs.len() as u32;
        write_u32(&mut writer, outputs_len)?;
        for output in &self.outputs {
            write_u32(&mut writer, output.witness_index())?;
        }

        // TODO: We use rmp_serde as its easier than doing it manually
        let buffer = rmp_serde::to_vec(&self.bytecode).unwrap();
        write_u32(&mut writer, buffer.len() as u32)?;
        write_bytes(&mut writer, &buffer)?;

        if let Some(pred) = &self.predicate {
            pred.write(&mut writer)?;
        }

        Ok(())
    }

    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let inputs_len = read_u32(&mut reader)?;
        let mut inputs = Vec::with_capacity(inputs_len as usize);
        for _ in 0..inputs_len {
            inputs.push(Expression::read(&mut reader)?);
        }

        let outputs_len = read_u32(&mut reader)?;
        let mut outputs = Vec::with_capacity(outputs_len as usize);
        for _ in 0..outputs_len {
            outputs.push(Witness(read_u32(&mut reader)?));
        }

        let buffer_len = read_u32(&mut reader)?;
        let mut buffer = vec![0u8; buffer_len as usize];
        reader.read_exact(&mut buffer)?;
        let opcodes: Vec<brillig_bytecode::Opcode> = rmp_serde::from_slice(&buffer).unwrap();

        // Read byte to figure out if there is a predicate
        let predicate_is_some = read_n::<1, _>(&mut reader)?[0] != 0;
        let predicate = match predicate_is_some {
            true => Some(Expression::read(&mut reader)?),
            false => None,
        };

        Ok(Brillig { inputs, outputs, bytecode: opcodes, predicate })
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Copy, Default)]
pub struct BlockId(pub u32);

/// Operation on a block
/// We can either write or read at a block index
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct MemOp {
    /// Can be 0 (read) or 1 (write)
    pub operation: Expression,
    pub index: Expression,
    pub value: Expression,
}

/// Represents operations on a block of length len of data
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryBlock {
    /// Id of the block
    pub id: BlockId,
    /// Length of the memory block
    pub len: u32,
    /// Trace of memory operations
    pub trace: Vec<MemOp>,
}

impl MemoryBlock {
    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let id = read_u32(&mut reader)?;
        let len = read_u32(&mut reader)?;
        let trace_len = read_u32(&mut reader)?;
        let mut trace = Vec::with_capacity(len as usize);
        for _i in 0..trace_len {
            let operation = Expression::read(&mut reader)?;
            let index = Expression::read(&mut reader)?;
            let value = Expression::read(&mut reader)?;
            trace.push(MemOp { operation, index, value });
        }
        Ok(MemoryBlock { id: BlockId(id), len, trace })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        write_u32(&mut writer, self.id.0)?;
        write_u32(&mut writer, self.len)?;
        write_u32(&mut writer, self.trace.len() as u32)?;

        for op in &self.trace {
            op.operation.write(&mut writer)?;
            op.index.write(&mut writer)?;
            op.value.write(&mut writer)?;
        }
        Ok(())
    }

    /// Returns the initialization vector of the MemoryBlock
    pub fn init_phase(&self) -> Vec<Expression> {
        let mut init = Vec::new();
        for i in 0..self.len as usize {
            assert_eq!(
                self.trace[i].operation,
                Expression::one(),
                "Block initialization require a write"
            );
            let index = self.trace[i]
                .index
                .to_const()
                .expect("Non-const index during Block initialization");
            if index != FieldElement::from(i as i128) {
                todo!(
                    "invalid index when initializing a block, we could try to sort the init phase"
                );
            }
            let value = self.trace[i].value.clone();
            assert!(value.is_degree_one_univariate(), "Block initialization requires a witness");
            init.push(value);
        }
        init
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleData {
    /// Name of the oracle
    pub name: String,
    /// Inputs
    pub inputs: Vec<Expression>,
    /// Input values - they are progressively computed by the pwg
    pub input_values: Vec<FieldElement>,
    /// Output witness
    pub outputs: Vec<Witness>,
    /// Output values - they are computed by the (external) oracle once the input_values are known
    pub output_values: Vec<FieldElement>,
}

impl OracleData {
    pub(crate) fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        let name_as_bytes = self.name.as_bytes();
        let name_len = name_as_bytes.len();
        write_u32(&mut writer, name_len as u32)?;
        write_bytes(&mut writer, name_as_bytes)?;

        let inputs_len = self.inputs.len() as u32;
        write_u32(&mut writer, inputs_len)?;
        for input in &self.inputs {
            input.write(&mut writer)?
        }

        let outputs_len = self.outputs.len() as u32;
        write_u32(&mut writer, outputs_len)?;
        for output in &self.outputs {
            write_u32(&mut writer, output.witness_index())?;
        }

        let inputs_len = self.input_values.len() as u32;
        write_u32(&mut writer, inputs_len)?;
        for input in &self.input_values {
            write_bytes(&mut writer, &input.to_be_bytes())?;
        }

        let outputs_len = self.output_values.len() as u32;
        write_u32(&mut writer, outputs_len)?;
        for output in &self.output_values {
            write_bytes(&mut writer, &output.to_be_bytes())?;
        }
        Ok(())
    }

    pub(crate) fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let name_len = read_u32(&mut reader)?;
        let name_as_bytes = read_bytes(&mut reader, name_len as usize)?;
        let name: String = String::from_utf8(name_as_bytes)
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

        let inputs_len = read_u32(&mut reader)?;
        let mut inputs = Vec::with_capacity(inputs_len as usize);
        for _ in 0..inputs_len {
            let input = Expression::read(&mut reader)?;
            inputs.push(input);
        }

        let outputs_len = read_u32(&mut reader)?;
        let mut outputs = Vec::with_capacity(outputs_len as usize);
        for _ in 0..outputs_len {
            let witness_index = read_u32(&mut reader)?;
            outputs.push(Witness(witness_index));
        }

        const FIELD_ELEMENT_NUM_BYTES: usize = FieldElement::max_num_bytes() as usize;
        let inputs_len = read_u32(&mut reader)?;
        let mut input_values = Vec::with_capacity(inputs_len as usize);
        for _ in 0..inputs_len {
            let value = read_field_element::<FIELD_ELEMENT_NUM_BYTES, _>(&mut reader)?;
            input_values.push(value);
        }

        let outputs_len = read_u32(&mut reader)?;
        let mut output_values = Vec::with_capacity(outputs_len as usize);
        for _ in 0..outputs_len {
            let value = read_field_element::<FIELD_ELEMENT_NUM_BYTES, _>(&mut reader)?;
            output_values.push(value);
        }

        Ok(OracleData { name, inputs, outputs, input_values, output_values })
    }
}

impl std::fmt::Display for OracleData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ORACLE: {}", self.name)?;
        let solved = if self.input_values.len() == self.inputs.len() { "solved" } else { "" };

        write!(
            f,
            "Inputs: _{}..._{}{solved}",
            self.inputs.first().unwrap(),
            self.inputs.last().unwrap()
        )?;

        let solved = if self.output_values.len() == self.outputs.len() { "solved" } else { "" };
        write!(
            f,
            "Outputs: _{}..._{}{solved}",
            self.outputs.first().unwrap().witness_index(),
            self.outputs.last().unwrap().witness_index()
        )
    }
}

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

impl Opcode {
    // TODO We can add a domain separator by doing something like:
    // TODO concat!("directive:", directive.name)
    pub fn name(&self) -> &str {
        match self {
            Opcode::Arithmetic(_) => "arithmetic",
            Opcode::Directive(directive) => directive.name(),
            Opcode::BlackBoxFuncCall(g) => g.name.name(),
            Opcode::Block(_) => "block",
            Opcode::RAM(_) => "ram",
            Opcode::ROM(_) => "rom",
            Opcode::Oracle(data) => &data.name,
            Opcode::Brillig(_) => "brillig",
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
            Opcode::Brillig { .. } => 7,
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
            Opcode::Brillig(brillig) => brillig.write(writer),
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
            7 => {
                let brillig = Brillig::read(reader)?;
                Ok(Opcode::Brillig(brillig))
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

impl BlackBoxFuncCall {
    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        write_u16(&mut writer, self.name.to_u16())?;

        let num_inputs = self.inputs.len() as u32;
        write_u32(&mut writer, num_inputs)?;

        for input in &self.inputs {
            write_u32(&mut writer, input.witness.witness_index())?;
            write_u32(&mut writer, input.num_bits)?;
        }

        let num_outputs = self.outputs.len() as u32;
        write_u32(&mut writer, num_outputs)?;

        for output in &self.outputs {
            write_u32(&mut writer, output.witness_index())?;
        }

        Ok(())
    }
    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let func_index = read_u16(&mut reader)?;
        let name = BlackBoxFunc::from_u16(func_index).ok_or(std::io::ErrorKind::InvalidData)?;

        let num_inputs = read_u32(&mut reader)?;
        let mut inputs = Vec::with_capacity(num_inputs as usize);
        for _ in 0..num_inputs {
            let witness = Witness(read_u32(&mut reader)?);
            let num_bits = read_u32(&mut reader)?;
            let input = FunctionInput { witness, num_bits };
            inputs.push(input)
        }

        let num_outputs = read_u32(&mut reader)?;
        let mut outputs = Vec::with_capacity(num_outputs as usize);
        for _ in 0..num_outputs {
            let witness = Witness(read_u32(&mut reader)?);
            outputs.push(witness)
        }

        Ok(BlackBoxFuncCall { name, inputs, outputs })
    }
}

impl std::fmt::Display for BlackBoxFuncCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uppercase_name: String = self.name.name().into();
        let uppercase_name = uppercase_name.to_uppercase();
        write!(f, "BLACKBOX::{uppercase_name} ")?;
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
                result +=
                    &format!("(_{}, num_bits: {})", inp.witness.witness_index(), inp.num_bits);
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
        write!(f, "{inputs_str}")?;
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
        write!(f, "{outputs_str}")?;
        write!(f, "]")
    }
}

impl std::fmt::Debug for BlackBoxFuncCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[test]
fn serialization_roundtrip() {
    fn read_write(opcode: Opcode) -> (Opcode, Opcode) {
        let mut bytes = Vec::new();
        opcode.write(&mut bytes).unwrap();
        let got_opcode = Opcode::read(&*bytes).unwrap();
        (opcode, got_opcode)
    }

    let opcode_arith = Opcode::Arithmetic(Expression::default());

    let opcode_black_box_func = Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
        name: BlackBoxFunc::AES,
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
