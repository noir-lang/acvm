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
use thiserror::Error;

use crate::native_types::Witness;

#[derive(Debug, Error)]
pub enum WitnessMapError {
    #[error(transparent)]
    MsgpackEncodeError(#[from] rmp_serde::encode::Error),

    #[error(transparent)]
    MsgpackDecodeError(#[from] rmp_serde::decode::Error),

    #[error(transparent)]
    DeflateError(#[from] std::io::Error),
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

impl TryFrom<WitnessMap> for Vec<u8> {
    type Error = WitnessMapError;

    fn try_from(val: WitnessMap) -> Result<Self, Self::Error> {
        let buf = rmp_serde::to_vec(&val)?;
        let mut deflater = DeflateEncoder::new(buf.as_slice(), Compression::best());
        let mut buf_c = Vec::new();
        deflater.read_to_end(&mut buf_c)?;
        Ok(buf_c)
    }
}

impl TryFrom<&[u8]> for WitnessMap {
    type Error = WitnessMapError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut deflater = DeflateDecoder::new(bytes);
        let mut buf_d = Vec::new();
        deflater.read_to_end(&mut buf_d)?;
        Ok(Self(rmp_serde::from_slice(buf_d.as_slice())?))
    }
}
