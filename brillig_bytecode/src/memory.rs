use serde::{Deserialize, Serialize};

/// Memory in the VM is used for storing arrays
///
/// ArrayIndex will be used to reference an Array element.
/// The pointer is needed to find it's location in memory,
/// and the index is used to offset from that point to find
/// the exact element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayIndex {
    pointer: usize,
    index: usize,
}
