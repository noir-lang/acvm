// The various passes that we can use over ACIR
pub mod optimizers;
pub mod transformers;

use crate::Language;
use acir::{
    circuit::{Circuit, Opcode},
    native_types::{Expression, Witness},
    BlackBoxFunc,
};
use indexmap::IndexMap;
use optimizers::GeneralOptimizer;
use thiserror::Error;
use transformers::{CSatTransformer, FallbackTransformer, R1CSTransformer};

use self::optimizers::RangeOptimizer;
use self::optimizers::Simplifier;

#[derive(PartialEq, Eq, Debug, Error)]
pub enum CompileError {
    #[error("The blackbox function {0} is not supported by the backend and acvm does not have a fallback implementation")]
    UnsupportedBlackBox(BlackBoxFunc),
}

pub fn compile(
    acir: Circuit,
    np_language: Language,
    is_opcode_supported: impl Fn(&Opcode) -> bool,
    simplifier: &Simplifier,
) -> Result<Circuit, CompileError> {
    // Instantiate the optimizer.
    // Currently the optimizer and reducer are one in the same
    // for CSAT

    // Fallback transformer pass
    let acir = FallbackTransformer::transform(acir, is_opcode_supported, simplifier)?;

    // General optimizer pass
    let mut opcodes: Vec<Opcode> = Vec::new();
    for opcode in acir.opcodes {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                opcodes.push(Opcode::Arithmetic(GeneralOptimizer::optimize(arith_expr)))
            }
            other_gate => opcodes.push(other_gate),
        };
    }
    let acir = Circuit { opcodes, ..acir };

    // Range optimization pass
    let range_optimizer = RangeOptimizer::new(acir);
    let acir = range_optimizer.replace_redundant_ranges();

    let transformer = match &np_language {
        crate::Language::R1CS => {
            let transformer = R1CSTransformer::new(acir);
            return Ok(transformer.transform());
        }
        crate::Language::PLONKCSat { width } => CSatTransformer::new(*width),
    };

    // TODO: the code below is only for CSAT transformer
    // TODO it may be possible to refactor it in a way that we do not need to return early from the r1cs
    // TODO or at the very least, we could put all of it inside of CSatOptimizer pass

    // Optimize the arithmetic gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut transformed_gates = Vec::new();

    let mut next_witness_index = acir.current_witness_index + 1;
    for opcode in acir.opcodes {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                let mut intermediate_variables: IndexMap<Witness, Expression> = IndexMap::new();

                let arith_expr = transformer.transform(
                    arith_expr,
                    &mut intermediate_variables,
                    next_witness_index,
                );

                // Update next_witness counter
                next_witness_index += intermediate_variables.len() as u32;
                let mut new_gates = Vec::new();
                for (_, mut g) in intermediate_variables {
                    g.sort();
                    new_gates.push(g);
                }
                new_gates.push(arith_expr);
                new_gates.sort();
                for gate in new_gates {
                    transformed_gates.push(Opcode::Arithmetic(gate));
                }
            }
            other_gate => transformed_gates.push(other_gate),
        }
    }

    let current_witness_index = next_witness_index - 1;

    Ok(Circuit {
        current_witness_index,
        opcodes: transformed_gates,
        // The optimizer does not add new public inputs
        public_parameters: acir.public_parameters,
        return_values: acir.return_values,
    })
}

#[test]
fn compile_api_compiles_with_backend() {
    use crate::ProofSystemCompiler;

    compile(
        Circuit::default(),
        Language::R1CS,
        |opcode| DummyProofSystem.opcode_supported(opcode),
        &Simplifier::new(0),
    )
    .unwrap();

    #[derive(Debug, Default)]
    struct DummyProofSystem;

    #[derive(Error, Debug)]
    enum DummyError {}

    impl ProofSystemCompiler for DummyProofSystem {
        type Error = DummyError;

        fn np_language(&self) -> Language {
            panic!("Path not trodden by this test")
        }

        fn opcode_supported(&self, _opcode: &Opcode) -> bool {
            panic!("Path not trodden by this test")
        }

        fn get_exact_circuit_size(&self, _circuit: &Circuit) -> Result<u32, Self::Error> {
            panic!("Path not trodden by this test")
        }

        fn preprocess(&self, _circuit: &Circuit) -> Result<(Vec<u8>, Vec<u8>), Self::Error> {
            panic!("Path not trodden by this test")
        }

        fn prove_with_pk(
            &self,
            _circuit: &Circuit,
            _witness_values: std::collections::BTreeMap<Witness, acir::FieldElement>,
            _proving_key: &[u8],
        ) -> Result<Vec<u8>, Self::Error> {
            panic!("Path not trodden by this test")
        }

        fn verify_with_vk(
            &self,
            _proof: &[u8],
            _public_inputs: std::collections::BTreeMap<Witness, acir::FieldElement>,
            _circuit: &Circuit,
            _verification_key: &[u8],
        ) -> Result<bool, Self::Error> {
            panic!("Path not trodden by this test")
        }
    }
}
