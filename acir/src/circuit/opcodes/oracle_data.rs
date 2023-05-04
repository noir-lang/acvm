use std::io::{Read, Write};

use crate::native_types::{Expression, Witness};
use crate::serialization::{read_bytes, read_field_element, read_u32, write_bytes, write_u32};
use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

        if !self.inputs.is_empty() {
            write!(
                f,
                "Inputs: _{}..._{}{solved}",
                self.inputs.first().unwrap(),
                self.inputs.last().unwrap()
            )?;
        }

        let solved = if self.output_values.len() == self.outputs.len() { "solved" } else { "" };
        write!(
            f,
            "Outputs: _{}..._{}{solved}",
            self.outputs.first().unwrap().witness_index(),
            self.outputs.last().unwrap().witness_index()
        )
    }
}
