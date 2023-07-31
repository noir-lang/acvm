// #![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

// TODO: Abscense of Per Pakckage targets
// https://doc.rust-lang.org/cargo/reference/unstable.html#per-package-target
// otherwise could be reorganized to make this file more pretty.

#[cfg(target_arch = "wasm32")]
mod barretenberg;
#[cfg(target_arch = "wasm32")]
mod build_info;
#[cfg(target_arch = "wasm32")]
mod compression;
#[cfg(target_arch = "wasm32")]
mod execute;
#[cfg(target_arch = "wasm32")]
mod foreign_call;
#[cfg(target_arch = "wasm32")]
mod js_witness_map;
#[cfg(target_arch = "wasm32")]
mod logging;
#[cfg(target_arch = "wasm32")]
mod public_witness;

#[cfg(target_arch = "wasm32")]
pub use build_info::build_info;
#[cfg(target_arch = "wasm32")]
pub use compression::{compress_witness, decompress_witness};
#[cfg(target_arch = "wasm32")]
pub use execute::execute_circuit;
#[cfg(target_arch = "wasm32")]
pub use js_witness_map::JsWitnessMap;
#[cfg(target_arch = "wasm32")]
pub use logging::{init_log_level, LogLevel};
#[cfg(target_arch = "wasm32")]
pub use public_witness::{get_public_parameters_witness, get_public_witness, get_return_witness};
