use std::io::{Read, Write};

use crate::native_types::Witness;
use crate::serialization::{read_u16, read_u32, write_u16, write_u32};
use crate::BlackBoxFunc;
use serde::{Deserialize, Serialize};

// Note: Some functions will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionInput {
    pub witness: Witness,
    pub num_bits: u32,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlackBoxFuncCall {
    #[allow(clippy::upper_case_acronyms)]
    AES {
        inputs: Vec<FunctionInput>,
        outputs: Vec<Witness>,
    },
    AND {
        lhs: FunctionInput,
        rhs: FunctionInput,
        output: Witness,
    },
    XOR {
        lhs: FunctionInput,
        rhs: FunctionInput,
        output: Witness,
    },
    RANGE {
        input: FunctionInput,
    },
    SHA256 {
        inputs: Vec<FunctionInput>,
        outputs: Vec<Witness>,
    },
    Blake2s {
        inputs: Vec<FunctionInput>,
        outputs: Vec<Witness>,
    },
    ComputeMerkleRoot {
        leaf: FunctionInput,
        index: FunctionInput,
        hash_path: Vec<FunctionInput>,
        output: Witness,
    },
    SchnorrVerify {
        public_key_x: FunctionInput,
        public_key_y: FunctionInput,
        signature: Vec<FunctionInput>,
        message: Vec<FunctionInput>,
        output: Witness,
    },
    Pedersen {
        inputs: Vec<FunctionInput>,
        outputs: Vec<Witness>,
    },
    // 128 here specifies that this function
    // should have 128 bits of security
    HashToField128Security {
        inputs: Vec<FunctionInput>,
        output: Witness,
    },
    EcdsaSecp256k1 {
        public_key_x: Vec<FunctionInput>,
        public_key_y: Vec<FunctionInput>,
        signature: Vec<FunctionInput>,
        message: Vec<FunctionInput>,
        output: Witness,
    },
    FixedBaseScalarMul {
        input: FunctionInput,
        outputs: Vec<Witness>,
    },
    Keccak256 {
        inputs: Vec<FunctionInput>,
        outputs: Vec<Witness>,
    },
}

fn write_input<W: Write>(input: &FunctionInput, mut writer: W) -> std::io::Result<()> {
    write_u32(&mut writer, input.witness.witness_index())?;
    write_u32(&mut writer, input.num_bits)?;
    Ok(())
}

fn write_inputs<W: Write>(inputs: &Vec<FunctionInput>, mut writer: W) -> std::io::Result<()> {
    let num_inputs = inputs.len() as u32;
    write_u32(&mut writer, num_inputs)?;

    for input in inputs {
        write_input(input, &mut writer)?;
    }

    Ok(())
}

fn write_outputs<W: Write>(outputs: &Vec<Witness>, mut writer: W) -> std::io::Result<()> {
    let num_inputs = outputs.len() as u32;
    write_u32(&mut writer, num_inputs)?;

    for output in outputs {
        write_u32(&mut writer, output.witness_index())?;
    }

    Ok(())
}

fn read_input<R: Read>(mut reader: R) -> std::io::Result<FunctionInput> {
    let witness_index = read_u32(&mut reader)?;
    let num_bits = read_u32(&mut reader)?;
    Ok(FunctionInput { witness: Witness::new(witness_index), num_bits })
}

fn read_inputs<R: Read>(mut reader: R) -> std::io::Result<Vec<FunctionInput>> {
    let num_inputs = read_u32(&mut reader)?;

    let mut inputs = Vec::with_capacity(num_inputs as usize);

    for _ in 0..num_inputs {
        inputs.push(read_input(&mut reader)?);
    }

    Ok(inputs)
}

fn read_outputs<R: Read>(mut reader: R) -> std::io::Result<Vec<Witness>> {
    let num_inputs = read_u32(&mut reader)?;

    let mut inputs = Vec::with_capacity(num_inputs as usize);

    for _ in 0..num_inputs {
        let witness_index = read_u32(&mut reader)?;
        inputs.push(Witness::new(witness_index));
    }

    Ok(inputs)
}

impl BlackBoxFuncCall {
    pub fn get_black_box_func(&self) -> BlackBoxFunc {
        match self {
            BlackBoxFuncCall::AES { .. } => BlackBoxFunc::AES,
            BlackBoxFuncCall::AND { .. } => BlackBoxFunc::AND,
            BlackBoxFuncCall::XOR { .. } => BlackBoxFunc::XOR,
            BlackBoxFuncCall::RANGE { .. } => BlackBoxFunc::RANGE,
            BlackBoxFuncCall::SHA256 { .. } => BlackBoxFunc::SHA256,
            BlackBoxFuncCall::Blake2s { .. } => BlackBoxFunc::Blake2s,
            BlackBoxFuncCall::ComputeMerkleRoot { .. } => BlackBoxFunc::ComputeMerkleRoot,
            BlackBoxFuncCall::SchnorrVerify { .. } => BlackBoxFunc::SchnorrVerify,
            BlackBoxFuncCall::Pedersen { .. } => BlackBoxFunc::Pedersen,
            BlackBoxFuncCall::HashToField128Security { .. } => BlackBoxFunc::HashToField128Security,
            BlackBoxFuncCall::EcdsaSecp256k1 { .. } => BlackBoxFunc::EcdsaSecp256k1,
            BlackBoxFuncCall::FixedBaseScalarMul { .. } => BlackBoxFunc::FixedBaseScalarMul,
            BlackBoxFuncCall::Keccak256 { .. } => BlackBoxFunc::Keccak256,
        }
    }

    pub fn get_inputs_vec(&self) -> Vec<FunctionInput> {
        match self {
            BlackBoxFuncCall::AES { inputs, .. }
            | BlackBoxFuncCall::SHA256 { inputs, .. }
            | BlackBoxFuncCall::Blake2s { inputs, .. }
            | BlackBoxFuncCall::Keccak256 { inputs, .. }
            | BlackBoxFuncCall::Pedersen { inputs, .. }
            | BlackBoxFuncCall::HashToField128Security { inputs, .. } => inputs.to_vec(),
            BlackBoxFuncCall::AND { lhs, rhs, .. } | BlackBoxFuncCall::XOR { lhs, rhs, .. } => {
                vec![*lhs, *rhs]
            }
            BlackBoxFuncCall::FixedBaseScalarMul { input, .. }
            | BlackBoxFuncCall::RANGE { input } => vec![*input],
            BlackBoxFuncCall::ComputeMerkleRoot { leaf, index, hash_path, .. } => {
                let mut inputs = Vec::new();
                inputs.push(*leaf);
                inputs.push(*index);
                inputs.extend(hash_path.iter().cloned());
                inputs
            }
            BlackBoxFuncCall::SchnorrVerify {
                public_key_x,
                public_key_y,
                signature,
                message,
                ..
            } => {
                let mut inputs = Vec::new();
                inputs.push(*public_key_x);
                inputs.push(*public_key_y);
                inputs.extend(signature.iter().cloned());
                inputs.extend(message.iter().cloned());
                inputs
            }
            BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x,
                public_key_y,
                signature,
                message,
                ..
            } => {
                let mut inputs = Vec::new();
                inputs.extend(public_key_x.iter().cloned());
                inputs.extend(public_key_y.iter().cloned());
                inputs.extend(signature.iter().cloned());
                inputs.extend(message.iter().cloned());
                inputs
            }
        }
    }

    pub fn get_outputs_vec(&self) -> Vec<Witness> {
        match self {
            BlackBoxFuncCall::AES { outputs, .. }
            | BlackBoxFuncCall::SHA256 { outputs, .. }
            | BlackBoxFuncCall::Blake2s { outputs, .. }
            | BlackBoxFuncCall::FixedBaseScalarMul { outputs, .. }
            | BlackBoxFuncCall::Pedersen { outputs, .. }
            | BlackBoxFuncCall::Keccak256 { outputs, .. } => outputs.to_vec(),
            BlackBoxFuncCall::AND { output, .. }
            | BlackBoxFuncCall::XOR { output, .. }
            | BlackBoxFuncCall::HashToField128Security { output, .. }
            | BlackBoxFuncCall::ComputeMerkleRoot { output, .. }
            | BlackBoxFuncCall::SchnorrVerify { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256k1 { output, .. } => vec![*output],
            BlackBoxFuncCall::RANGE { .. } => vec![],
        }
    }

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        write_u16(&mut writer, self.get_black_box_func().to_u16())?;

        write_inputs(&self.get_inputs_vec(), &mut writer)?;
        write_outputs(&self.get_outputs_vec(), &mut writer)?;

        Ok(())
    }

    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let func_index = read_u16(&mut reader)?;
        let name = BlackBoxFunc::from_u16(func_index).ok_or(std::io::ErrorKind::InvalidData)?;

        let inputs = read_inputs(&mut reader)?;
        let outputs = read_outputs(&mut reader)?;

        let func_call = match name {
            BlackBoxFunc::AES => BlackBoxFuncCall::AES { inputs, outputs },
            BlackBoxFunc::AND => {
                let lhs = inputs[0];
                let rhs = inputs[1];
                let output = outputs[0];
                BlackBoxFuncCall::AND { lhs, rhs, output }
            }
            BlackBoxFunc::XOR => {
                let lhs = inputs[0];
                let rhs = inputs[1];
                let output = outputs[0];
                BlackBoxFuncCall::XOR { lhs, rhs, output }
            }
            BlackBoxFunc::RANGE => {
                let input = inputs[0];
                BlackBoxFuncCall::RANGE { input }
            }
            BlackBoxFunc::SHA256 => BlackBoxFuncCall::SHA256 { inputs, outputs },
            BlackBoxFunc::Blake2s => BlackBoxFuncCall::Blake2s { inputs, outputs },
            BlackBoxFunc::ComputeMerkleRoot => BlackBoxFuncCall::ComputeMerkleRoot {
                leaf: inputs[0],
                index: inputs[1],
                hash_path: inputs[2..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::SchnorrVerify => BlackBoxFuncCall::SchnorrVerify {
                public_key_x: inputs[0],
                public_key_y: inputs[1],
                signature: inputs[2..66].to_vec(),
                message: inputs[66..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::Pedersen => BlackBoxFuncCall::Pedersen { inputs, outputs },
            BlackBoxFunc::HashToField128Security => {
                BlackBoxFuncCall::HashToField128Security { inputs, output: outputs[0] }
            }
            BlackBoxFunc::EcdsaSecp256k1 => BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x: inputs[0..32].to_vec(),
                public_key_y: inputs[32..64].to_vec(),
                signature: inputs[64..128].to_vec(),
                message: inputs[128..].to_vec(),
                output: outputs[0],
            },
            BlackBoxFunc::FixedBaseScalarMul => {
                BlackBoxFuncCall::FixedBaseScalarMul { input: inputs[0], outputs }
            }
            BlackBoxFunc::Keccak256 => BlackBoxFuncCall::Keccak256 { inputs, outputs },
        };

        Ok(func_call)
    }
}

const ABBREVIATION_LIMIT: usize = 5;

fn get_inputs_string(inputs: &Vec<FunctionInput>) -> String {
    // Once a vectors length gets above this limit,
    // instead of listing all of their elements, we use ellipses
    // to abbreviate them
    let should_abbreviate_inputs = inputs.len() <= ABBREVIATION_LIMIT;

    if should_abbreviate_inputs {
        let mut result = String::new();
        for (index, inp) in inputs.iter().enumerate() {
            result += &format!("(_{}, num_bits: {})", inp.witness.witness_index(), inp.num_bits);
            // Add a comma, unless it is the last entry
            if index != inputs.len() - 1 {
                result += ", "
            }
        }
        result
    } else {
        let first = inputs.first().unwrap();
        let last = inputs.last().unwrap();

        let mut result = String::new();

        result += &format!(
            "(_{}, num_bits: {})...(_{}, num_bits: {})",
            first.witness.witness_index(),
            first.num_bits,
            last.witness.witness_index(),
            last.num_bits,
        );

        result
    }
}

fn get_outputs_string(outputs: &Vec<Witness>) -> String {
    let should_abbreviate_outputs = outputs.len() <= ABBREVIATION_LIMIT;

    if should_abbreviate_outputs {
        let mut result = String::new();
        for (index, output) in outputs.iter().enumerate() {
            result += &format!("_{}", output.witness_index());
            // Add a comma, unless it is the last entry
            if index != outputs.len() - 1 {
                result += ", "
            }
        }
        result
    } else {
        let first = outputs.first().unwrap();
        let last = outputs.last().unwrap();

        let mut result = String::new();
        result += &format!("(_{},...,_{})", first.witness_index(), last.witness_index());
        result
    }
}

impl std::fmt::Display for BlackBoxFuncCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uppercase_name: String = self.get_black_box_func().name().into();
        let uppercase_name = uppercase_name.to_uppercase();
        write!(f, "BLACKBOX::{uppercase_name} ")?;
        // INPUTS
        write!(f, "[")?;

        let inputs_str = get_inputs_string(&self.get_inputs_vec());

        write!(f, "{inputs_str}")?;
        write!(f, "] ")?;

        // OUTPUTS
        write!(f, "[ ")?;

        let outputs_str = get_outputs_string(&self.get_outputs_vec());

        write!(f, "{outputs_str}")?;

        write!(f, "]")
    }
}

impl std::fmt::Debug for BlackBoxFuncCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
