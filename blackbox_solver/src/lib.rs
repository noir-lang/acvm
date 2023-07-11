#![warn(unused_crate_dependencies)]
#![warn(unreachable_pub)]

use acir::BlackBoxFunc;
use acir_field::FieldElement;
use blake2::digest::generic_array::GenericArray;
use blake2::{Blake2s256, Digest};
use sha2::Sha256;
use sha3::Keccak256;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum BlackBoxResolutionError {
    #[error("unsupported blackbox function: {0}")]
    Unsupported(BlackBoxFunc),
    #[error("failed to solve blackbox function: {0}, reason: {1}")]
    Failed(BlackBoxFunc, String),
}

/// This component will generate outputs for [`Opcode::BlackBoxFuncCall`] where the underlying [`acir::BlackBoxFunc`]
/// doesn't have a canonical Rust implementation.
///
/// Returns an [`BlackBoxResolutionError`] if the backend does not support the given [`Opcode::BlackBoxFuncCall`].
pub trait BlackBoxFunctionSolver {
    fn schnorr_verify(
        &self,
        public_key_x: &FieldElement,
        public_key_y: &FieldElement,
        signature: &[u8],
        message: &[u8],
    ) -> Result<bool, BlackBoxResolutionError>;
    fn pedersen(
        &self,
        inputs: &[FieldElement],
        domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError>;
    fn fixed_base_scalar_mul(
        &self,
        input: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError>;
}

pub fn sha256(inputs: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError> {
    generic_hash_256::<Sha256>(inputs)
        .map_err(|err| BlackBoxResolutionError::Failed(BlackBoxFunc::SHA256, err))
}

pub fn blake2s(inputs: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError> {
    generic_hash_256::<Blake2s256>(inputs)
        .map_err(|err| BlackBoxResolutionError::Failed(BlackBoxFunc::Blake2s, err))
}

pub fn keccak256(inputs: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError> {
    generic_hash_256::<Keccak256>(inputs)
        .map_err(|err| BlackBoxResolutionError::Failed(BlackBoxFunc::Keccak256, err))
}

pub fn hash_to_field_128_security(inputs: &[u8]) -> Result<FieldElement, BlackBoxResolutionError> {
    generic_hash_to_field::<Blake2s256>(inputs)
        .map_err(|err| BlackBoxResolutionError::Failed(BlackBoxFunc::HashToField128Security, err))
}

pub fn ecdsa_secp256k1_verify(
    hashed_msg: &[u8],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    Ok(verify_secp256k1_ecdsa_signature(hashed_msg, public_key_x, public_key_y, signature))
}

pub fn ecdsa_secp256r1_verify(
    hashed_msg: &[u8],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    Ok(verify_secp256r1_ecdsa_signature(hashed_msg, public_key_x, public_key_y, signature))
}

/// Does a generic hash of the inputs returning the resulting 32 bytes as fields.
fn generic_hash_256<D: Digest>(message: &[u8]) -> Result<[u8; 32], String> {
    let output_bytes: [u8; 32] =
        D::digest(message).as_slice().try_into().map_err(|_| "digest should be 256 bits")?;

    Ok(output_bytes)
}

/// Does a generic hash of the entire inputs converting the resulting hash into a single output field.
fn generic_hash_to_field<D: Digest>(message: &[u8]) -> Result<FieldElement, String> {
    let output_bytes: [u8; 32] =
        D::digest(message).as_slice().try_into().map_err(|_| "digest should be 256 bits")?;

    Ok(FieldElement::from_be_bytes_reduce(&output_bytes))
}

// TODO(https://github.com/noir-lang/acvm/issues/402): remove from here and use the one from acvm
fn verify_secp256k1_ecdsa_signature(
    hashed_msg: &[u8],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> bool {
    use k256::elliptic_curve::sec1::FromEncodedPoint;
    use k256::elliptic_curve::PrimeField;

    use k256::{ecdsa::Signature, Scalar};
    use k256::{
        elliptic_curve::{
            sec1::{Coordinates, ToEncodedPoint},
            IsHigh,
        },
        AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
    };
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

// TODO(https://github.com/noir-lang/acvm/issues/402): remove from here and use the one from acvm
fn verify_secp256r1_ecdsa_signature(
    hashed_msg: &[u8],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> bool {
    use p256::elliptic_curve::sec1::FromEncodedPoint;
    use p256::elliptic_curve::PrimeField;

    use p256::{ecdsa::Signature, Scalar};
    use p256::{
        elliptic_curve::{
            sec1::{Coordinates, ToEncodedPoint},
            IsHigh,
        },
        AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
    };

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
