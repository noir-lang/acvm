pub mod black_box_functions;
pub mod directives;
pub mod opcodes;
pub use opcodes::Opcode;

use crate::native_types::Witness;
use crate::serialization::{read_u32, write_u32};
use rmp_serde;
use serde::{Deserialize, Serialize};

use flate2::bufread::{DeflateDecoder, DeflateEncoder};
use flate2::Compression;
use std::collections::BTreeSet;
use std::io::prelude::*;

const VERSION_NUMBER: u32 = 0;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Circuit {
    // current_witness_index is the highest witness index in the circuit. The next witness to be added to this circuit
    // will take on this value. (The value is cached here as an optimization.)
    pub current_witness_index: u32,
    pub opcodes: Vec<Opcode>,

    // ACIR distinguishes between the public inputs which are provided externally or calculated within the circuit and returned.
    // The elements of these sets may not be mutually exclusive, i.e. a parameter may be returned from the circuit.
    // All public inputs (parameters and return values) must be provided to the verifier at verification time.
    /// The set of public inputs provided by the prover.
    pub public_parameters: PublicInputs,
    /// The set of public inputs calculated within the circuit.
    pub return_values: PublicInputs,
}

impl Circuit {
    pub fn num_vars(&self) -> u32 {
        self.current_witness_index + 1
    }

    /// Returns all public inputs. This includes those provided as parameters to the circuit and those
    /// computed as return values.
    pub fn public_inputs(&self) -> PublicInputs {
        let public_inputs =
            self.public_parameters.0.union(&self.return_values.0).cloned().collect();
        PublicInputs(public_inputs)
    }

    #[deprecated(
        note = "we want to use a serialization strategy that is easy to implement in many languages (without ffi). use `read` instead"
    )]
    pub fn from_bytes(bytes: &[u8]) -> Circuit {
        let mut deflater = DeflateDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).unwrap();
        rmp_serde::from_slice(buf_d.as_slice()).unwrap()
    }

    #[deprecated(
        note = "we want to use a serialization strategy that is easy to implement in many languages (without ffi).use `write` instead"
    )]
    pub fn to_bytes(&self) -> Vec<u8> {
        let buf = rmp_serde::to_vec(&self).unwrap();
        let mut deflater = DeflateEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).unwrap();
        buf_c
    }

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        write_u32(&mut writer, VERSION_NUMBER)?;

        write_u32(&mut writer, self.current_witness_index)?;

        let public_input_indices = self.public_parameters.indices();
        write_u32(&mut writer, public_input_indices.len() as u32)?;
        for public_input_index in public_input_indices {
            write_u32(&mut writer, public_input_index)?;
        }

        let public_output_indices = self.return_values.indices();
        write_u32(&mut writer, public_output_indices.len() as u32)?;
        for public_output_index in public_output_indices {
            write_u32(&mut writer, public_output_index)?;
        }

        write_u32(&mut writer, self.opcodes.len() as u32)?;
        for opcode in &self.opcodes {
            opcode.write(&mut writer)?;
        }
        Ok(())
    }
    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let version_number = read_u32(&mut reader)?;
        // TODO (Note): we could use semver versioning from the Cargo.toml
        // here and then reject anything that has a major bump
        //
        // We may also not want to do that if we do not want to couple serialization
        // with other breaking changes
        if version_number != VERSION_NUMBER {
            return Err(std::io::ErrorKind::InvalidData.into());
        }

        let current_witness_index = read_u32(&mut reader)?;

        let num_public_parameters = read_u32(&mut reader)?;
        let mut public_parameters = PublicInputs(BTreeSet::new());
        for _ in 0..num_public_parameters {
            let public_parameter_index = Witness(read_u32(&mut reader)?);
            public_parameters.0.insert(public_parameter_index);
        }
        let num_return_values = read_u32(&mut reader)?;
        let mut return_values = PublicInputs(BTreeSet::new());
        for _ in 0..num_return_values {
            let return_value_index = Witness(read_u32(&mut reader)?);
            return_values.0.insert(return_value_index);
        }

        let num_opcodes = read_u32(&mut reader)?;
        let mut opcodes = Vec::with_capacity(num_opcodes as usize);
        for _ in 0..num_opcodes {
            let opcode = Opcode::read(&mut reader)?;
            opcodes.push(opcode)
        }

        Ok(Self { current_witness_index, opcodes, public_parameters, return_values })
    }
}

impl std::fmt::Display for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "current witness index : {}", self.current_witness_index)?;

        let write_public_inputs = |f: &mut std::fmt::Formatter<'_>,
                                   public_inputs: &PublicInputs|
         -> Result<(), std::fmt::Error> {
            write!(f, "[")?;
            let public_input_indices = public_inputs.indices();
            for (index, public_input) in public_input_indices.iter().enumerate() {
                write!(f, "{public_input}")?;
                if index != public_input_indices.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            writeln!(f, "]")
        };

        write!(f, "public parameters indices : ")?;
        write_public_inputs(f, &self.public_parameters)?;

        write!(f, "return value indices : ")?;
        write_public_inputs(f, &self.return_values)?;

        for opcode in &self.opcodes {
            writeln!(f, "{opcode}")?
        }
        Ok(())
    }
}

impl std::fmt::Debug for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PublicInputs(pub BTreeSet<Witness>);

impl PublicInputs {
    /// Returns the witness index of each public input
    pub fn indices(&self) -> Vec<u32> {
        self.0.iter().map(|witness| witness.witness_index()).collect()
    }

    pub fn contains(&self, index: usize) -> bool {
        self.0.contains(&Witness(index as u32))
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::{
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Circuit, Opcode, PublicInputs,
    };
    use crate::native_types::{Expression, Witness};
    use acir_field::FieldElement;

    fn and_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
            name: crate::BlackBoxFunc::AND,
            inputs: vec![
                FunctionInput { witness: Witness(1), num_bits: 4 },
                FunctionInput { witness: Witness(2), num_bits: 4 },
            ],
            outputs: vec![Witness(3)],
        })
    }
    fn range_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
            name: crate::BlackBoxFunc::RANGE,
            inputs: vec![FunctionInput { witness: Witness(1), num_bits: 8 }],
            outputs: vec![],
        })
    }
    fn oracle_opcode() -> Opcode {
        Opcode::Oracle {
            name: String::from("oracle-name"),
            inputs: vec![Expression {
                mul_terms: vec![(FieldElement::from(123u128), Witness(1), Witness(2))],
                linear_combinations: vec![(FieldElement::from(456u128), Witness(34))],
                q_c: FieldElement::from(12345678u128),
            }],
            outputs: vec![Witness(1), Witness(2), Witness(3)],
        }
    }

    #[test]
    fn serialization_roundtrip() {
        let circuit = Circuit {
            current_witness_index: 5,
            opcodes: vec![and_opcode(), range_opcode(), oracle_opcode()],
            public_parameters: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(12)])),
            return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(4), Witness(12)])),
        };

        fn read_write(circuit: Circuit) -> (Circuit, Circuit) {
            let mut bytes = Vec::new();
            circuit.write(&mut bytes).unwrap();
            let got_circuit = Circuit::read(&*bytes).unwrap();
            (circuit, got_circuit)
        }

        let (circ, got_circ) = read_write(circuit);
        assert_eq!(circ, got_circ)
    }

    #[test]
    fn test_serialize() {
        let circuit = Circuit {
            current_witness_index: 0,
            opcodes: vec![
                Opcode::Arithmetic(crate::native_types::Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from(8u128),
                }),
                range_opcode(),
                and_opcode(),
                oracle_opcode(),
            ],
            public_parameters: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
            return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
        };

        let json = serde_json::to_string_pretty(&circuit).unwrap();

        let deserialized = serde_json::from_str(&json).unwrap();
        assert_eq!(circuit, deserialized);
    }

    #[allow(deprecated)]
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
                oracle_opcode(),
            ],
            public_parameters: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
            return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
        };

        let bytes = circuit.to_bytes();

        let deserialized = Circuit::from_bytes(bytes.as_slice());
        assert_eq!(circuit, deserialized);
    }
}
