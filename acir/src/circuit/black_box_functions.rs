use serde::{Deserialize, Serialize};
#[cfg(test)]
use strum_macros::EnumIter;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(EnumIter))]
pub enum BlackBoxFunc {
    #[allow(clippy::upper_case_acronyms)]
    AES128,
    AND,
    XOR,
    RANGE,
    SHA256,
    Blake2s,
    SchnorrVerify,
    Pedersen,
    // 128 here specifies that this function
    // should have 128 bits of security
    HashToField128Security,
    EcdsaSecp256k1,
    FixedBaseScalarMul,
    Keccak256,
}

impl std::fmt::Display for BlackBoxFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl BlackBoxFunc {
    pub fn name(&self) -> &'static str {
        match self {
            BlackBoxFunc::AES128 => "aes128",
            BlackBoxFunc::SHA256 => "sha256",
            BlackBoxFunc::SchnorrVerify => "schnorr_verify",
            BlackBoxFunc::Blake2s => "blake2s",
            BlackBoxFunc::Pedersen => "pedersen",
            BlackBoxFunc::HashToField128Security => "hash_to_field_128_security",
            BlackBoxFunc::EcdsaSecp256k1 => "ecdsa_secp256k1",
            BlackBoxFunc::FixedBaseScalarMul => "fixed_base_scalar_mul",
            BlackBoxFunc::AND => "and",
            BlackBoxFunc::XOR => "xor",
            BlackBoxFunc::RANGE => "range",
            BlackBoxFunc::Keccak256 => "keccak256",
        }
    }
    pub fn lookup(op_name: &str) -> Option<BlackBoxFunc> {
        match op_name {
            "aes128" => Some(BlackBoxFunc::AES128),
            "sha256" => Some(BlackBoxFunc::SHA256),
            "schnorr_verify" => Some(BlackBoxFunc::SchnorrVerify),
            "blake2s" => Some(BlackBoxFunc::Blake2s),
            "pedersen" => Some(BlackBoxFunc::Pedersen),
            "hash_to_field_128_security" => Some(BlackBoxFunc::HashToField128Security),
            "ecdsa_secp256k1" => Some(BlackBoxFunc::EcdsaSecp256k1),
            "fixed_base_scalar_mul" => Some(BlackBoxFunc::FixedBaseScalarMul),
            "and" => Some(BlackBoxFunc::AND),
            "xor" => Some(BlackBoxFunc::XOR),
            "range" => Some(BlackBoxFunc::RANGE),
            "keccak256" => Some(BlackBoxFunc::Keccak256),
            _ => None,
        }
    }
    pub fn is_valid_black_box_func_name(op_name: &str) -> bool {
        BlackBoxFunc::lookup(op_name).is_some()
    }
}

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

    use crate::BlackBoxFunc;

    #[test]
    fn consistent_function_names() {
        for bb_func in BlackBoxFunc::iter() {
            let resolved_func = BlackBoxFunc::lookup(bb_func.name()).unwrap_or_else(|| {
                panic!("BlackBoxFunc::lookup couldn't find black box function {}", bb_func)
            });
            assert_eq!(
                resolved_func, bb_func,
                "BlackBoxFunc::lookup returns unexpected BlackBoxFunc"
            )
        }
    }
}
