use crate::native_types::Witness;
use crate::BlackBoxFunc;
use serde::{Deserialize, Serialize};

// Note: Some functions will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionInput {
    pub witness: Witness,
    pub num_bits: u32,
}

impl FunctionInput {
    pub fn dummy() -> Self {
        Self { witness: Witness(0), num_bits: 0 }
    }
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
        hashed_message: Vec<FunctionInput>,
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
    VerifyProof {
        key: Vec<FunctionInput>,
        proof: Vec<FunctionInput>,
        public_inputs: Vec<FunctionInput>,
        key_hash: FunctionInput,
        input_aggregation_object: Vec<FunctionInput>,
        // This is the recursive verification output aggregation object.
        // The name `outputs` was kept to simplify code reuse with the other BlackBoxFuncCall's
        outputs: Vec<Witness>,
    },
}

impl BlackBoxFuncCall {
    pub fn dummy(bb_func: BlackBoxFunc) -> Self {
        match bb_func {
            BlackBoxFunc::AES => BlackBoxFuncCall::AES { inputs: vec![], outputs: vec![] },
            BlackBoxFunc::AND => BlackBoxFuncCall::AND {
                lhs: FunctionInput::dummy(),
                rhs: FunctionInput::dummy(),
                output: Witness(0),
            },
            BlackBoxFunc::XOR => BlackBoxFuncCall::XOR {
                lhs: FunctionInput::dummy(),
                rhs: FunctionInput::dummy(),
                output: Witness(0),
            },
            BlackBoxFunc::RANGE => BlackBoxFuncCall::RANGE { input: FunctionInput::dummy() },
            BlackBoxFunc::SHA256 => BlackBoxFuncCall::SHA256 { inputs: vec![], outputs: vec![] },
            BlackBoxFunc::Blake2s => BlackBoxFuncCall::Blake2s { inputs: vec![], outputs: vec![] },
            BlackBoxFunc::SchnorrVerify => BlackBoxFuncCall::SchnorrVerify {
                public_key_x: FunctionInput::dummy(),
                public_key_y: FunctionInput::dummy(),
                signature: vec![],
                message: vec![],
                output: Witness(0),
            },
            BlackBoxFunc::Pedersen => {
                BlackBoxFuncCall::Pedersen { inputs: vec![], outputs: vec![] }
            }
            BlackBoxFunc::HashToField128Security => {
                BlackBoxFuncCall::HashToField128Security { inputs: vec![], output: Witness(0) }
            }
            BlackBoxFunc::EcdsaSecp256k1 => BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x: vec![],
                public_key_y: vec![],
                signature: vec![],
                hashed_message: vec![],
                output: Witness(0),
            },
            BlackBoxFunc::FixedBaseScalarMul => BlackBoxFuncCall::FixedBaseScalarMul {
                input: FunctionInput::dummy(),
                outputs: vec![],
            },
            BlackBoxFunc::Keccak256 => {
                BlackBoxFuncCall::Keccak256 { inputs: vec![], outputs: vec![] }
            }
            BlackBoxFunc::VerifyProof => BlackBoxFuncCall::VerifyProof {
                key: vec![],
                proof: vec![],
                public_inputs: vec![],
                key_hash: FunctionInput::dummy(),
                input_aggregation_object: vec![],
                outputs: vec![],
            },
        }
    }

    pub fn get_black_box_func(&self) -> BlackBoxFunc {
        match self {
            BlackBoxFuncCall::AES { .. } => BlackBoxFunc::AES,
            BlackBoxFuncCall::AND { .. } => BlackBoxFunc::AND,
            BlackBoxFuncCall::XOR { .. } => BlackBoxFunc::XOR,
            BlackBoxFuncCall::RANGE { .. } => BlackBoxFunc::RANGE,
            BlackBoxFuncCall::SHA256 { .. } => BlackBoxFunc::SHA256,
            BlackBoxFuncCall::Blake2s { .. } => BlackBoxFunc::Blake2s,
            BlackBoxFuncCall::SchnorrVerify { .. } => BlackBoxFunc::SchnorrVerify,
            BlackBoxFuncCall::Pedersen { .. } => BlackBoxFunc::Pedersen,
            BlackBoxFuncCall::HashToField128Security { .. } => BlackBoxFunc::HashToField128Security,
            BlackBoxFuncCall::EcdsaSecp256k1 { .. } => BlackBoxFunc::EcdsaSecp256k1,
            BlackBoxFuncCall::FixedBaseScalarMul { .. } => BlackBoxFunc::FixedBaseScalarMul,
            BlackBoxFuncCall::Keccak256 { .. } => BlackBoxFunc::Keccak256,
            BlackBoxFuncCall::VerifyProof { .. } => BlackBoxFunc::VerifyProof,
        }
    }

    pub fn name(&self) -> &str {
        self.get_black_box_func().name()
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
            BlackBoxFuncCall::SchnorrVerify {
                public_key_x,
                public_key_y,
                signature,
                message,
                ..
            } => {
                let mut inputs = Vec::with_capacity(2 + signature.len() + message.len());
                inputs.push(*public_key_x);
                inputs.push(*public_key_y);
                inputs.extend(signature.iter().copied());
                inputs.extend(message.iter().copied());
                inputs
            }
            BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                ..
            } => {
                let mut inputs = Vec::with_capacity(
                    public_key_x.len()
                        + public_key_y.len()
                        + signature.len()
                        + hashed_message.len(),
                );
                inputs.extend(public_key_x.iter().copied());
                inputs.extend(public_key_y.iter().copied());
                inputs.extend(signature.iter().copied());
                inputs.extend(hashed_message.iter().copied());
                inputs
            }
            BlackBoxFuncCall::VerifyProof {
                key,
                proof,
                public_inputs,
                key_hash,
                input_aggregation_object,
                ..
            } => {
                let mut inputs = Vec::with_capacity(
                    key.len()
                        + proof.len()
                        + public_inputs.len()
                        + 1
                        + input_aggregation_object.len(),
                );
                inputs.extend(key.iter().copied());
                inputs.extend(proof.iter().copied());
                inputs.extend(public_inputs.iter().copied());
                inputs.push(*key_hash);
                // If we do not have an input aggregation object assigned
                // do not return it as part of the input vector as to not trigger
                if !input_aggregation_object.iter().any(|v| v.witness == Witness(0)) {
                    inputs.extend(input_aggregation_object.iter().copied());
                }
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
            | BlackBoxFuncCall::Keccak256 { outputs, .. }
            | BlackBoxFuncCall::VerifyProof { outputs, .. } => outputs.to_vec(),
            BlackBoxFuncCall::AND { output, .. }
            | BlackBoxFuncCall::XOR { output, .. }
            | BlackBoxFuncCall::HashToField128Security { output, .. }
            | BlackBoxFuncCall::SchnorrVerify { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256k1 { output, .. } => vec![*output],
            BlackBoxFuncCall::RANGE { .. } => vec![],
        }
    }
}

const ABBREVIATION_LIMIT: usize = 5;

fn get_inputs_string(inputs: &[FunctionInput]) -> String {
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

fn get_outputs_string(outputs: &[Witness]) -> String {
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
        let uppercase_name = self.name().to_uppercase();
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
