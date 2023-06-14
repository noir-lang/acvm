mod general;
mod redundant_range;
mod simplify;

pub(crate) use general::GeneralOptimizer;
pub(crate) use redundant_range::RangeOptimizer;
// Public as these need to be passed to `acvm::compiler::compile()`
pub(crate) use simplify::CircuitSimplifier; // TODO(AD) re-public internals for now
