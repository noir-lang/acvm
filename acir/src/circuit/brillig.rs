use crate::native_types::{Expression, Witness};
use brillig_vm::ForeignCallResult;
use serde::{Deserialize, Serialize};

/// Inputs for the Brillig VM. These are the initial inputs
/// that the Brillig VM will use to start.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub enum BrilligInputs {
    Single(Expression),
    Array(Vec<Expression>),
}

/// Outputs for the Brillig VM. Once the VM has completed
/// execution, this will be the object that is returned.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub enum BrilligOutputs {
    Simple(Witness),
    Array(Vec<Witness>),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Brillig {
    pub inputs: Vec<BrilligInputs>,
    pub outputs: Vec<BrilligOutputs>,
    /// Results of oracles/functions external to brillig like a database read.
    // Each element of this vector corresponds to a single foreign call but may contain several values.
    pub foreign_call_results: Vec<ForeignCallResult>,
    /// The Brillig VM bytecode to be executed by this ACIR opcode.
    pub bytecode: Vec<brillig_vm::Opcode>,
    /// Predicate of the Brillig execution - indicates if it should be skipped
    pub predicate: Option<Expression>,
}

impl Brillig {
    /// Canonically hashes the Brillig struct.
    ///
    /// Some Brillig instances may or may not be resolved, so we do
    /// not hash the `foreign_call_results`.
    pub fn canonical_hash(&self) -> u64 {
        let mut serialize_vector = rmp_serde::to_vec(&self.inputs).unwrap();
        serialize_vector.extend(rmp_serde::to_vec(&self.outputs).unwrap());
        serialize_vector.extend(rmp_serde::to_vec(&self.bytecode).unwrap());
        serialize_vector.extend(rmp_serde::to_vec(&self.predicate).unwrap());

        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        hasher.write(&serialize_vector);
        hasher.finish()
    }
}
