mod general;
mod redundant_range;
pub mod simplify;

pub(crate) use general::GeneralOpt as GeneralOptimizer;
pub(crate) use redundant_range::RangeOptimizer;
pub(crate) use simplify::CircuitSimplifier as Simplifier;
