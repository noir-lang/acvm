pub mod blackbox_functions;
pub mod directives;
pub mod opcodes;
pub use opcodes::Opcode;

use crate::native_types::Witness;
use rmp_serde;
use serde::{Deserialize, Serialize};

use flate2::bufread::{DeflateDecoder, DeflateEncoder};
use flate2::Compression;
use std::io::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Circuit {
    pub current_witness_index: u32,
    pub opcodes: Vec<Opcode>,
    pub public_inputs: PublicInputs,
}

impl Circuit {
    pub fn num_vars(&self) -> u32 {
        self.current_witness_index + 1
    }

    pub fn from_bytes(bytes: &[u8]) -> Circuit {
        let mut deflater = DeflateDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).unwrap();
        rmp_serde::from_slice(buf_d.as_slice()).unwrap()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let buf = rmp_serde::to_vec(&self).unwrap();
        let mut deflater = DeflateEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).unwrap();
        buf_c
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicInputs(pub Vec<Witness>);

impl PublicInputs {
    /// Returns the witness index of each public input
    pub fn indices(&self) -> Vec<u32> {
        self.0
            .iter()
            .map(|witness| witness.witness_index())
            .collect()
    }

    pub fn contains(&self, index: usize) -> bool {
        self.0.contains(&Witness(index as u32))
    }
}

#[cfg(test)]
mod test {
    use super::{
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Circuit, Opcode, PublicInputs,
    };
    use crate::native_types::Witness;
    use acir_field::FieldElement;

    fn and_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
            name: crate::BlackBoxFunc::AND,
            inputs: vec![
                FunctionInput {
                    witness: Witness(1),
                    num_bits: 4,
                },
                FunctionInput {
                    witness: Witness(2),
                    num_bits: 4,
                },
            ],
            outputs: vec![Witness(3)],
        })
    }
    fn range_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
            name: crate::BlackBoxFunc::RANGE,
            inputs: vec![FunctionInput {
                witness: Witness(1),
                num_bits: 8,
            }],
            outputs: vec![],
        })
    }

    #[test]
    fn test_serialize() {
        let circuit = Circuit {
            current_witness_index: 0,
            opcodes: vec![
                Opcode::Arithmetic(crate::native_types::Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from_hex("FFFF").unwrap(),
                }),
                range_opcode(),
                and_opcode(),
            ],
            public_inputs: PublicInputs(vec![Witness(2)]),
        };

        let json = serde_json::to_string_pretty(&circuit).unwrap();
        println!("serialized: {}", json);

        let deserialized = serde_json::from_str(&json).unwrap();
        assert_eq!(circuit, deserialized);
    }

    #[test]
    fn test_to_byte() {
        let circuit = Circuit {
            current_witness_index: 0,
            opcodes: vec![
                Opcode::Arithmetic(crate::native_types::Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from_hex("FFFF").unwrap(),
                }),
                range_opcode(),
                and_opcode(),
            ],
            public_inputs: PublicInputs(vec![Witness(2)]),
        };

        let bytes = circuit.to_bytes();
        println!("bytes: {:?}", bytes);

        let deserialized = Circuit::from_bytes(bytes.as_slice());
        assert_eq!(circuit, deserialized);
    }
}
