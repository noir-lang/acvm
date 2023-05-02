use std::{
    collections::{btree_map, BTreeMap},
    io::Read,
    ops::Index,
};

use acir_field::FieldElement;
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
pub struct WitnessMap(BTreeMap<Witness, FieldElement>);

impl WitnessMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn get(&self, witness: &Witness) -> Option<&FieldElement> {
        self.0.get(witness)
    }
    pub fn get_index(&self, index: u32) -> Option<&FieldElement> {
        self.0.get(&index.into())
    }
    pub fn contains_key(&self, key: &Witness) -> bool {
        self.0.contains_key(key)
    }
    pub fn insert(&mut self, key: Witness, value: FieldElement) -> Option<FieldElement> {
        self.0.insert(key, value)
    }
}

impl Index<&Witness> for WitnessMap {
    type Output = FieldElement;

    fn index(&self, index: &Witness) -> &Self::Output {
        &self.0[index]
    }
}

pub struct IntoIter(btree_map::IntoIter<Witness, FieldElement>);

impl Iterator for IntoIter {
    type Item = (Witness, FieldElement);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl IntoIterator for WitnessMap {
    type Item = (Witness, FieldElement);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl From<BTreeMap<Witness, FieldElement>> for WitnessMap {
    fn from(value: BTreeMap<Witness, FieldElement>) -> Self {
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
