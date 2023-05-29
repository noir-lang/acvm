pub mod black_box_functions;
pub mod brillig;
pub mod directives;
pub mod opcodes;

use crate::native_types::Witness;
pub use opcodes::Opcode;

use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::prelude::*;

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

    pub fn write<W: std::io::Write>(&self, writer: W) -> std::io::Result<()> {
        let buf = rmp_serde::to_vec(&self).unwrap();
        let mut deflater = DeflateEncoder::new(writer, Compression::best());
        deflater.write_all(&buf).unwrap();

        Ok(())
    }
    pub fn read<R: std::io::Read>(reader: R) -> std::io::Result<Self> {
        let mut deflater = DeflateDecoder::new(reader);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).unwrap();
        let circuit = rmp_serde::from_slice(buf_d.as_slice()).unwrap();
        Ok(circuit)
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
        opcodes::{BlackBoxFuncCall, FunctionInput, OracleData},
        Circuit, Opcode, PublicInputs,
    };
    use crate::native_types::{Expression, Witness};
    use acir_field::FieldElement;

    fn directive_opcode() -> Opcode {
        Opcode::Directive(super::directives::Directive::Invert {
            x: Witness(0),
            result: Witness(1),
        })
    }
    fn and_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput { witness: Witness(1), num_bits: 4 },
            rhs: FunctionInput { witness: Witness(2), num_bits: 4 },
            output: Witness(3),
        })
    }
    fn range_opcode() -> Opcode {
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: Witness(1), num_bits: 8 },
        })
    }
    fn oracle_opcode() -> Opcode {
        Opcode::Oracle(OracleData {
            name: String::from("oracle-name"),
            inputs: vec![Expression {
                mul_terms: vec![(FieldElement::from(123u128), Witness(1), Witness(2))],
                linear_combinations: vec![(FieldElement::from(456u128), Witness(34))],
                q_c: FieldElement::from(12345678u128),
            }],
            input_values: vec![],
            outputs: vec![Witness(1), Witness(2), Witness(3)],
            output_values: vec![],
        })
    }

    #[test]
    fn serialization_roundtrip() {
        let circuit = Circuit {
            current_witness_index: 5,
            opcodes: vec![and_opcode(), range_opcode(), oracle_opcode(), directive_opcode()],
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
}
