#![warn(unused_crate_dependencies)]
#![warn(unreachable_pub)]

pub mod compiler;
pub mod pwg;

use base64::Engine;
pub use blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};
use core::fmt::Debug;
use pwg::OpcodeResolutionError;

// re-export acir
pub use acir;
pub use acir::FieldElement;
// re-export brillig vm
pub use brillig_vm;
// re-export blackbox solver
pub use blackbox_solver;

/// Supported NP complete languages
/// This might need to be in ACIR instead
#[derive(Debug, Clone, Copy)]
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}

#[test]
fn decode_me() {
    let string = "H4sIAAAAAAAA/+3WU6wdWwCA4X17z6lxatu2bds6tW3btm3btnlt9Vrle9N/0j/NeelTmzRpupIvf2b2zswkKytrPQuFQtFDL0e4HWg/QjR8jDB/D/4bAzERC7ERB3ERD/GRABFIiERIjCRIimRIjhRIiVRIjTRIi3RIjwzIiEzIjCzIimzIjhzIiVzIjTzIi3zIjwIoiEIojCIoimIojhIoiVIojTIoi3IojwqoiEqojCqoimqojhqoiVqojTqoi3qojwZoiEZojCZoimZojhZoiVZojTZoi0i0Q3t0QEd0Qmd0QVd0Q3f0QE/0Qm/0QV/0Q38McC7DnctI53UQBmMIhmIYhmMERmIURmMMxmIcxmMCJmISJmMKpmIapmMGZmIWZmMO5mIe5mMBFmIRFmMJlmIZlmMFVmIVVmMN1mId1mMDNmITNmMLtmIbtmMHdmIXdmMP9mIf9uMADuIQDuMIjuIYjuMETuIUTuMMzuIczuMCLuISLuMKruIaruMGbuIWbuMO7uIe7jsP0ZyLYHzivWB9RXjvM3yOL/AlvsLX+Abf4jt8jx/wI37Cz/gFD3x2sA6jruHnvve57woaZsNtdBvDxrSxbGwbx8a18Wx8m8BG2IQ2kU1sk9ikNplNblPYlDaVTW3T2LQ2nU1vM9iMNpPNbLPYrDabzW5z2Jw2l81t89i8Np/NbwvYgraQLWyL2KK2mC1uS9iStpQtbcvYsracLW8r2Iq2kq1sq9iqtpqtbmvYmraWrW3r2Lq2nq1vG9iGtpFtbJvYpraZbW5b2Ja2lW1t29i2NtK2s+1tB9vRdrKdbRfb1Xaz3W0P29P2sr1tH9vX9rP97QA7MMp3BmOQ14PtEDvUDrPD7Qg70o6yo+0YO9aOs+PtBDvRTrKT7RQ71U6z0+0MO9POsrPtHDvXzrPz7QK70C6yi+0Su9Qus8vtCrvSrrKr7Rq71q6z6+0Gu9FuspvtFrvVbrPb7Q670+6yu+0eu9fus/vtAXvQHrKH7RF71B6zx+0Je9KesqftGXvWnrPn7QV70V6yl+0Ve9Ves9ftDXvT3rK37R17196z923UPS+4/tS+2vh+xW/4HX/gT/yFv/EP/sV/+B8P8QiP8QRPQy83srC3+LwHPufDIfj9OAR/OPS+m0NvsOCDxf6mB9vXjRcamcakwQ4AAA==";
    let circuit_bytes = base64::engine::general_purpose::STANDARD.decode(string).unwrap();
    dbg!(circuit_bytes);
    panic!("foo");
}
