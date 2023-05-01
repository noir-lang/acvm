use std::io::{Read, Write};

use crate::native_types::Witness;
use crate::serialization::{read_u16, read_u32, write_u16, write_u32};
use crate::BlackBoxFunc;
use serde::{Deserialize, Serialize};

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
