use crate::{memory::Memory, opcodes::HeapVector, HeapArray, RegisterIndex, Registers, Value};
use acir_field::FieldElement;
use blake2::digest::generic_array::GenericArray;
use blake2::{Blake2s256, Digest};
use k256::elliptic_curve::sec1::FromEncodedPoint;
use k256::elliptic_curve::PrimeField;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sha3::Keccak256;

use k256::{ecdsa::Signature, Scalar};
use k256::{
    elliptic_curve::{
        sec1::{Coordinates, ToEncodedPoint},
        IsHigh,
    },
    AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
};

/// Converts an array of Values to an array of u8s.
fn to_u8_vec(inputs: &[Value]) -> Vec<u8> {
    let mut result = Vec::with_capacity(inputs.len());
    for input in inputs {
        let field_bytes = input.to_field().to_be_bytes();
        let byte = field_bytes.last().unwrap();
        result.push(*byte);
    }
    result
}

/// Does a generic hash of the inputs storing the resulting 32 bytes as items in the output array.
fn generic_hash_256<D: Digest>(
    message: &HeapVector,
    output: &HeapArray,
    registers: &mut Registers,
    memory: &mut Memory,
) {
    let message_values = memory.read_slice(
        registers.get(message.pointer).to_usize(),
        registers.get(message.size).to_usize(),
    );
    let message_bytes = to_u8_vec(message_values);

    assert!(output.size == 32, "Expected a 32-element result array");

    let output_bytes: [u8; 32] =
        D::digest(message_bytes).as_slice().try_into().expect("digest should be 256 bits");
    let output_values: Vec<Value> = output_bytes.iter().map(|b| (*b as u128).into()).collect();

    memory.write_slice(registers.get(output.pointer).to_usize(), &output_values);
}

/// Does a generic hash of the inputs storing the resulting hash into the output register.
fn generic_hash_to_field<D: Digest>(
    message: &HeapVector,
    output: &RegisterIndex,
    registers: &mut Registers,
    memory: &mut Memory,
) {
    let message_values = memory.read_slice(
        registers.get(message.pointer).to_usize(),
        registers.get(message.size).to_usize(),
    );
    let message_bytes = to_u8_vec(message_values);

    let output_bytes: [u8; 32] =
        D::digest(message_bytes).as_slice().try_into().expect("digest should be 256 bits");

    let reduced_res = FieldElement::from_be_bytes_reduce(&output_bytes);

    registers.set(*output, reduced_res.into());
}

// TODO: remove from here and use the one from acvm
fn verify_secp256k1_ecdsa_signature(
    hashed_msg: &[u8],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> bool {
    // Convert the inputs into k256 data structures

    let signature = Signature::try_from(signature.as_slice()).unwrap();

    let point = EncodedPoint::from_affine_coordinates(
        public_key_x_bytes.into(),
        public_key_y_bytes.into(),
        true,
    );
    let pubkey = PublicKey::from_encoded_point(&point).unwrap();

    let z = Scalar::from_repr(*GenericArray::from_slice(hashed_msg)).unwrap();

    // Finished converting bytes into data structures

    let r = signature.r();
    let s = signature.s();

    // Ensure signature is "low S" normalized ala BIP 0062
    if s.is_high().into() {
        return false;
    }

    let s_inv = s.invert().unwrap();
    let u1 = z * s_inv;
    let u2 = *r * s_inv;

    #[allow(non_snake_case)]
    let R: AffinePoint = ((ProjectivePoint::GENERATOR * u1)
        + (ProjectivePoint::from(*pubkey.as_affine()) * u2))
        .to_affine();

    match R.to_encoded_point(false).coordinates() {
        Coordinates::Uncompressed { x, y: _ } => Scalar::from_repr(*x).unwrap().eq(&r),
        _ => unreachable!("Point is uncompressed"),
    }
}

/// These opcodes provide an equivalent of ACIR blackbox functions.
/// They are implemented as native functions in the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlackBoxOp {
    /// Calculates the SHA256 hash of the inputs.
    Sha256 { message: HeapVector, output: HeapArray },
    /// Calculates the Blake2s hash of the inputs.
    Blake2s { message: HeapVector, output: HeapArray },
    /// Calculates the Keccak256 hash of the inputs.
    Keccak256 { message: HeapVector, output: HeapArray },
    /// Hashes a set of inputs and applies the field modulus to the result
    /// to return a value which can be represented as a [`FieldElement`][acir_field::FieldElement]
    ///
    /// This is implemented using the `Blake2s` hash function.
    /// The "128" in the name specifies that this function should have 128 bits of security.
    HashToField128Security { message: HeapVector, output: RegisterIndex },
    /// Verifies a ECDSA signature over the secp256k1 curve.
    EcdsaSecp256k1 {
        hashed_msg: HeapVector,
        public_key_x: HeapArray,
        public_key_y: HeapArray,
        signature: HeapArray,
        result: RegisterIndex,
    },
}

impl BlackBoxOp {
    pub(crate) fn evaluate(&self, registers: &mut Registers, memory: &mut Memory) {
        match self {
            BlackBoxOp::Sha256 { message, output } => {
                generic_hash_256::<Sha256>(message, output, registers, memory);
            }
            BlackBoxOp::Blake2s { message, output } => {
                generic_hash_256::<Blake2s256>(message, output, registers, memory);
            }
            BlackBoxOp::Keccak256 { message, output } => {
                generic_hash_256::<Keccak256>(message, output, registers, memory);
            }
            BlackBoxOp::HashToField128Security { message, output } => {
                generic_hash_to_field::<Blake2s256>(message, output, registers, memory);
            }
            BlackBoxOp::EcdsaSecp256k1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result: result_register,
            } => {
                let message_values = memory.read_slice(
                    registers.get(hashed_msg.pointer).to_usize(),
                    registers.get(hashed_msg.size).to_usize(),
                );
                let message_bytes = to_u8_vec(message_values);

                let public_key_x_bytes: [u8; 32] =
                    to_u8_vec(memory.read_slice(
                        registers.get(public_key_x.pointer).to_usize(),
                        public_key_x.size,
                    ))
                    .try_into()
                    .expect("Expected a 32-element public key x array");

                let public_key_y_bytes: [u8; 32] =
                    to_u8_vec(memory.read_slice(
                        registers.get(public_key_y.pointer).to_usize(),
                        public_key_y.size,
                    ))
                    .try_into()
                    .expect("Expected a 32-element public key y array");

                let signature_bytes: [u8; 64] = to_u8_vec(
                    memory.read_slice(registers.get(signature.pointer).to_usize(), signature.size),
                )
                .try_into()
                .expect("Expected a 64-element signature array");

                let result = verify_secp256k1_ecdsa_signature(
                    &message_bytes,
                    &public_key_x_bytes,
                    &public_key_y_bytes,
                    &signature_bytes,
                );

                registers.set(*result_register, (result as u128).into())
            }
        }
    }
}
