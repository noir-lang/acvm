use serde::{Deserialize, Serialize};
#[cfg(test)]
use strum_macros::EnumIter;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(EnumIter))]
pub enum BlackBoxFunc {
    #[allow(clippy::upper_case_acronyms)]
    AES,
    AND,
    XOR,
    RANGE,
    SHA256,
    Blake2s,
    MerkleMembership,
    SchnorrVerify,
    Pedersen,
    HashToField,
    EcdsaSecp256k1,
    FixedBaseScalarMul,
}

impl std::fmt::Display for BlackBoxFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl BlackBoxFunc {
    pub fn to_u16(self) -> u16 {
        match self {
            BlackBoxFunc::AES => 0,
            BlackBoxFunc::SHA256 => 1,
            BlackBoxFunc::MerkleMembership => 2,
            BlackBoxFunc::SchnorrVerify => 3,
            BlackBoxFunc::Blake2s => 4,
            BlackBoxFunc::Pedersen => 5,
            BlackBoxFunc::HashToField => 6,
            BlackBoxFunc::EcdsaSecp256k1 => 7,
            BlackBoxFunc::FixedBaseScalarMul => 8,
            BlackBoxFunc::AND => 9,
            BlackBoxFunc::XOR => 10,
            BlackBoxFunc::RANGE => 11,
        }
    }
    pub fn from_u16(index: u16) -> Option<Self> {
        let function = match index {
            0 => BlackBoxFunc::AES,
            1 => BlackBoxFunc::SHA256,
            2 => BlackBoxFunc::MerkleMembership,
            3 => BlackBoxFunc::SchnorrVerify,
            4 => BlackBoxFunc::Blake2s,
            5 => BlackBoxFunc::Pedersen,
            6 => BlackBoxFunc::HashToField,
            7 => BlackBoxFunc::EcdsaSecp256k1,
            8 => BlackBoxFunc::FixedBaseScalarMul,
            9 => BlackBoxFunc::AND,
            10 => BlackBoxFunc::XOR,
            11 => BlackBoxFunc::RANGE,
            _ => return None,
        };
        Some(function)
    }
    pub fn name(&self) -> &'static str {
        match self {
            BlackBoxFunc::AES => "aes",
            BlackBoxFunc::SHA256 => "sha256",
            BlackBoxFunc::MerkleMembership => "merkle_membership",
            BlackBoxFunc::SchnorrVerify => "schnorr_verify",
            BlackBoxFunc::Blake2s => "blake2s",
            BlackBoxFunc::Pedersen => "pedersen",
            BlackBoxFunc::HashToField => "hash_to_field",
            BlackBoxFunc::EcdsaSecp256k1 => "ecdsa_secp256k1",
            BlackBoxFunc::FixedBaseScalarMul => "fixed_base_scalar_mul",
            BlackBoxFunc::AND => "and",
            BlackBoxFunc::XOR => "xor",
            BlackBoxFunc::RANGE => "range",
        }
    }
    pub fn lookup(op_name: &str) -> Option<BlackBoxFunc> {
        match op_name {
            "aes" => Some(BlackBoxFunc::AES),
            "sha256" => Some(BlackBoxFunc::SHA256),
            "merkle_membership" => Some(BlackBoxFunc::MerkleMembership),
            "schnorr_verify" => Some(BlackBoxFunc::SchnorrVerify),
            "blake2s" => Some(BlackBoxFunc::Blake2s),
            "pedersen" => Some(BlackBoxFunc::Pedersen),
            "hash_to_field" => Some(BlackBoxFunc::HashToField),
            "ecdsa_secp256k1" => Some(BlackBoxFunc::EcdsaSecp256k1),
            "fixed_base_scalar_mul" => Some(BlackBoxFunc::FixedBaseScalarMul),
            "and" => Some(BlackBoxFunc::AND),
            "xor" => Some(BlackBoxFunc::XOR),
            "range" => Some(BlackBoxFunc::RANGE),
            _ => None,
        }
    }
    pub fn is_valid_black_box_func_name(op_name: &str) -> bool {
        BlackBoxFunc::lookup(op_name).is_some()
    }
    pub fn definition(&self) -> FuncDefinition {
        let name = self.name();
        match self {
            BlackBoxFunc::AES => unimplemented!(),
            BlackBoxFunc::SHA256 => FuncDefinition {
                name,
                input_size: InputSize::Variable,
                output_size: OutputSize(32),
            },
            BlackBoxFunc::Blake2s => FuncDefinition {
                name,
                input_size: InputSize::Variable,
                output_size: OutputSize(32),
            },
            BlackBoxFunc::HashToField => FuncDefinition {
                name,
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            BlackBoxFunc::MerkleMembership => FuncDefinition {
                name,
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            BlackBoxFunc::SchnorrVerify => FuncDefinition {
                name,
                // XXX: input_size can be changed to fixed, once we hash
                // the message before passing it to schnorr.
                // This is assuming all hashes will be 256 bits. Reasonable?
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            BlackBoxFunc::Pedersen => FuncDefinition {
                name,
                input_size: InputSize::Variable,
                output_size: OutputSize(2),
            },
            BlackBoxFunc::EcdsaSecp256k1 => FuncDefinition {
                name,
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            BlackBoxFunc::FixedBaseScalarMul => FuncDefinition {
                name,
                input_size: InputSize::Fixed(1),
                output_size: OutputSize(2),
            },
            BlackBoxFunc::AND => FuncDefinition {
                name,
                input_size: InputSize::Fixed(2),
                output_size: OutputSize(1),
            },
            BlackBoxFunc::XOR => FuncDefinition {
                name,
                input_size: InputSize::Fixed(2),
                output_size: OutputSize(1),
            },
            BlackBoxFunc::RANGE => FuncDefinition {
                name,
                input_size: InputSize::Fixed(1),
                output_size: OutputSize(0),
            },
        }
    }
}

// Descriptor as to whether the input/output is fixed or variable
// Example: The input for Sha256 is Variable and the output is fixed at 2 witnesses
// each holding 128 bits of the actual Sha256 function
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum InputSize {
    Variable,
    Fixed(u128),
}

impl InputSize {
    pub fn fixed_size(&self) -> Option<u128> {
        match self {
            InputSize::Variable => None,
            InputSize::Fixed(size) => Some(*size),
        }
    }
}

// Output size Cannot currently vary, so we use a separate struct
// XXX: In the future, we may be able to allow the output to vary based on the input size, however this implies support for dynamic circuits
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OutputSize(pub u128);

#[derive(Clone, Debug, Hash)]
// Specs for how many inputs/outputs the method takes.
pub struct FuncDefinition {
    pub name: &'static str,
    pub input_size: InputSize,
    pub output_size: OutputSize,
}

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

    use crate::BlackBoxFunc;

    #[test]
    fn consistent_function_names() {
        for bb_func in BlackBoxFunc::iter() {
            let resolved_func = BlackBoxFunc::lookup(bb_func.name()).unwrap_or_else(|| {
                panic!(
                    "BlackBoxFunc::lookup couldn't find black box function {}",
                    bb_func
                )
            });
            assert_eq!(
                resolved_func, bb_func,
                "BlackBoxFunc::lookup returns unexpected BlackBoxFunc"
            )
        }
    }
    #[test]
    fn consistent_index() {
        for bb_func in BlackBoxFunc::iter() {
            let func_index = bb_func.to_u16();
            let got_bb_func =
                BlackBoxFunc::from_u16(func_index).expect("blackbox function should have an index");
            assert_eq!(
                got_bb_func, bb_func,
                "BlackBox function index lookup is inconsistent"
            )
        }
    }
}
