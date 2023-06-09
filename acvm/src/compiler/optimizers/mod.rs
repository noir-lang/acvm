mod general;
mod redundant_range;
mod simplify;

pub(crate) use general::GeneralOptimizer;
pub(crate) use redundant_range::RangeOptimizer;
pub use simplify::{CircuitSimplifier as Simplifier, SimplifyResult};
