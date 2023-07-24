pub mod black_box_functions;
pub mod brillig;
pub mod directives;
pub mod opcodes;

use crate::native_types::Witness;
pub use opcodes::Opcode;

#[cfg(feature = "serialize-messagepack")]
use flate2::{read::DeflateDecoder, write::DeflateEncoder};
use std::io::prelude::*;

#[cfg(not(feature = "serialize-messagepack"))]
use flate2::write::GzEncoder;
use flate2::Compression;

// use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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

#[cfg(test)]
mod reflection {
    use std::{
        fs::File,
        io::Write,
        path::{Path, PathBuf},
    };

    use brillig::{
        BinaryFieldOp, BinaryIntOp, BlackBoxOp, BrilligOpcode, ForeignCallOutput, RegisterOrMemory,
    };
    use serde_reflection::{Tracer, TracerConfig};

    use crate::{
        circuit::{
            brillig::{BrilligInputs, BrilligOutputs},
            directives::{Directive, LogInfo},
            opcodes::BlackBoxFuncCall,
            Circuit, Opcode,
        },
        native_types::{Witness, WitnessMap},
    };

    #[test]
    fn serde_acir_cpp_codegen() {
        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<Circuit>().unwrap();
        tracer.trace_simple_type::<Opcode>().unwrap();
        tracer.trace_simple_type::<BinaryFieldOp>().unwrap();
        tracer.trace_simple_type::<BlackBoxFuncCall>().unwrap();
        tracer.trace_simple_type::<BrilligInputs>().unwrap();
        tracer.trace_simple_type::<BrilligOutputs>().unwrap();
        tracer.trace_simple_type::<BrilligOpcode>().unwrap();
        tracer.trace_simple_type::<BinaryIntOp>().unwrap();
        tracer.trace_simple_type::<BlackBoxOp>().unwrap();
        tracer.trace_simple_type::<Directive>().unwrap();
        tracer.trace_simple_type::<ForeignCallOutput>().unwrap();
        tracer.trace_simple_type::<RegisterOrMemory>().unwrap();
        tracer.trace_simple_type::<LogInfo>().unwrap();

        let registry = tracer.registry().unwrap();

        let data = serde_json::to_vec(&registry).unwrap();
        write_to_file(&data, &PathBuf::from("./acir.json"));

        // Create C++ class definitions.
        let mut source = Vec::new();
        let config = serde_generate::CodeGeneratorConfig::new("Circuit".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, &registry).unwrap();

        write_to_file(&source, &PathBuf::from("./acir.cpp"));
    }

    #[test]
    fn serde_witnessmap_cpp_codegen() {
        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<Witness>().unwrap();
        tracer.trace_simple_type::<WitnessMap>().unwrap();

        let registry = tracer.registry().unwrap();

        let data = serde_json::to_vec(&registry).unwrap();
        write_to_file(&data, &PathBuf::from("./witness.json"));

        // Create C++ class definitions.
        let mut source = Vec::new();
        let config = serde_generate::CodeGeneratorConfig::new("WitnessMap".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, &registry).unwrap();

        write_to_file(&source, &PathBuf::from("./witness.cpp"));
    }

    pub(super) fn write_to_file(bytes: &[u8], path: &Path) -> String {
        let display = path.display();

        let mut file = match File::create(path) {
            Err(why) => panic!("couldn't create {display}: {why}"),
            Ok(file) => file,
        };

        match file.write_all(bytes) {
            Err(why) => panic!("couldn't write to {display}: {why}"),
            Ok(_) => display.to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
/// Opcodes are given labels so that callers can
/// map opcodes to debug information related to their context.
pub enum OpcodeLabel {
    #[default]
    Unresolved,
    Resolved(u64),
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

    #[cfg(feature = "serialize-messagepack")]
    pub fn write<W: std::io::Write>(&self, writer: W) -> std::io::Result<()> {
        let buf = rmp_serde::to_vec(&self).unwrap();
        let mut deflater = flate2::write::DeflateEncoder::new(writer, Compression::best());
        deflater.write_all(&buf).unwrap();

        Ok(())
    }
    #[cfg(feature = "serialize-messagepack")]
    pub fn read<R: std::io::Read>(reader: R) -> std::io::Result<Self> {
        let mut deflater = flate2::read::DeflateDecoder::new(reader);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).unwrap();
        let circuit = rmp_serde::from_slice(buf_d.as_slice()).unwrap();
        Ok(circuit)
    }

    #[cfg(not(feature = "serialize-messagepack"))]
    pub fn write<W: std::io::Write>(&self, writer: W) -> std::io::Result<()> {
        let buf = bincode::serde::encode_to_vec(&self, bincode::config::standard()).unwrap();
        let mut encoder = GzEncoder::new(writer, Compression::default());
        encoder.write_all(&buf).unwrap();
        Ok(())
    }

    #[cfg(not(feature = "serialize-messagepack"))]
    pub fn read<R: std::io::Read>(reader: R) -> std::io::Result<Self> {
        let mut gz_decoder = flate2::read::GzDecoder::new(reader);
        let mut buf_d = Vec::new();
        gz_decoder.read_to_end(&mut buf_d).unwrap();
        let (circuit, _len): (Circuit, usize) =
            bincode::serde::decode_from_slice(buf_d.as_slice(), bincode::config::standard())
                .unwrap();
        Ok(circuit)
    }

    /// Initial list of labels attached to opcodes.
    pub fn initial_opcode_labels(&self) -> Vec<OpcodeLabel> {
        (0..self.opcodes.len()).map(|label| OpcodeLabel::Resolved(label as u64)).collect()
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
mod tests {
    use std::collections::BTreeSet;

    use super::{
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Circuit, Opcode, PublicInputs,
    };
    use crate::native_types::Witness;
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

    #[test]
    fn serialization_roundtrip() {
        let circuit = Circuit {
            current_witness_index: 5,
            opcodes: vec![and_opcode(), range_opcode(), directive_opcode()],
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
            ],
            public_parameters: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
            return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2)])),
        };

        let json = serde_json::to_string_pretty(&circuit).unwrap();

        let deserialized = serde_json::from_str(&json).unwrap();
        assert_eq!(circuit, deserialized);
    }
}
