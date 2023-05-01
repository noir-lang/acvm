use std::{collections::BTreeMap, io::Read, ops::Index};

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
}

impl From<u32> for Witness {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// A map from the witnesses in a constraint system to the field element values
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct WitnessMap(BTreeMap<Witness, acir_field::FieldElement>);

impl WitnessMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn get(&self, witness: &Witness) -> Option<&acir_field::FieldElement> {
        self.0.get(witness)
    }
    pub fn get_index(&self, index: u32) -> Option<&acir_field::FieldElement> {
        self.0.get(&index.into())
    }
    pub fn contains_key(&self, key: &Witness) -> bool {
        self.0.contains_key(key)
    }
    pub fn insert(
        &mut self,
        key: Witness,
        value: acir_field::FieldElement,
    ) -> Option<acir_field::FieldElement> {
        self.0.insert(key, value)
    }
}

impl Index<&Witness> for WitnessMap {
    type Output = acir_field::FieldElement;

    fn index(&self, index: &Witness) -> &Self::Output {
        &self.0[index]
    }
}

impl From<BTreeMap<Witness, acir_field::FieldElement>> for WitnessMap {
    fn from(value: BTreeMap<Witness, acir_field::FieldElement>) -> Self {
        Self(value)
    }
}

impl From<WitnessMap> for Vec<u8> {
    fn from(val: WitnessMap) -> Self {
        let buf = rmp_serde::to_vec(&val).unwrap();
        let mut deflater = DeflateEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c).unwrap();
        buf_c
    }
}

impl From<&[u8]> for WitnessMap {
    fn from(bytes: &[u8]) -> Self {
        let mut deflater = DeflateDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d).unwrap();
        Self(rmp_serde::from_slice(buf_d.as_slice()).unwrap())
    }
}

impl From<WitnessMap> for Vec<acir_field::FieldElement> {
    fn from(val: WitnessMap) -> Self {
        val.0.into_values().collect()
    }
}
