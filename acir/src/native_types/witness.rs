use std::io::Read;

use flate2::{
    bufread::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use serde::{Deserialize, Serialize};

// Witness might be a misnomer. This is an index that represents the position a witness will take
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct Witness(pub u32);

impl Witness {
    pub fn new(witness_index: u32) -> Witness {
        Witness(witness_index)
    }
    pub fn witness_index(&self) -> u32 {
        self.0
    }
    pub fn as_usize(&self) -> usize {
        // This is safe as long as the architecture is 32bits minimum
        self.0 as usize
    }

    pub const fn can_defer_constraint(&self) -> bool {
        true
    }

    #[deprecated = "ACIR no longer specifies how a witness map should be serialized."]
    pub fn to_bytes(
        witnesses: &std::collections::BTreeMap<Witness, acir_field::FieldElement>,
    ) -> Vec<u8> {
        let buf = rmp_serde::to_vec(witnesses).unwrap();
        let mut deflater = DeflateEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).unwrap();
        buf_c
    }

    #[deprecated = "ACIR no longer specifies how a witness map should be serialized."]
    pub fn from_bytes(
        bytes: &[u8],
    ) -> std::collections::BTreeMap<Witness, acir_field::FieldElement> {
        let mut deflater = DeflateDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).unwrap();
        rmp_serde::from_slice(buf_d.as_slice()).unwrap()
    }
}
