use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    FieldElement,
};
use blake2::digest::generic_array::GenericArray;
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

use crate::{
    pwg::{insert_value, OpcodeResolution},
    OpcodeResolutionError,
};

use super::to_u8_vec;

pub(crate) fn secp256k1_prehashed(
    initial_witness: &mut WitnessMap,
    public_key_x_inputs: &[FunctionInput],
    public_key_y_inputs: &[FunctionInput],
    signature_inputs: &[FunctionInput],
    hashed_message_inputs: &[FunctionInput],
    output: Witness,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hashed_message = to_u8_vec(initial_witness, hashed_message_inputs)?;

    // These errors should never be emitted in practice as they would imply malformed ACIR generation.
    let pub_key_x: [u8; 32] =
        to_u8_vec(initial_witness, public_key_x_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256k1,
                format!("expected pubkey_x size 32 but received {}", public_key_x_inputs.len()),
            )
        })?;

    let pub_key_y: [u8; 32] =
        to_u8_vec(initial_witness, public_key_y_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256k1,
                format!("expected pubkey_y size 32 but received {}", public_key_y_inputs.len()),
            )
        })?;

    let signature: [u8; 64] =
        to_u8_vec(initial_witness, signature_inputs)?.try_into().map_err(|_| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                acir::BlackBoxFunc::EcdsaSecp256k1,
                format!("expected signature size 64 but received {}", signature_inputs.len()),
            )
        })?;

    let is_valid =
        verify_secp256k1_ecdsa_signature(&hashed_message, &pub_key_x, &pub_key_y, &signature);

    insert_value(&output, FieldElement::from(is_valid), initial_witness)?;
    Ok(OpcodeResolution::Solved)
}

/// Verify an ECDSA signature over the secp256k1 elliptic curve, given the hashed message
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

#[cfg(test)]
mod test {
    use super::verify_secp256k1_ecdsa_signature;

    #[test]
    fn verifies_valid_signature_with_low_s_value() {
        // 0x3a73f4123a5cd2121f21cd7e8d358835476949d035d9c2da6806b4633ac8c1e2,
        let hashed_message: [u8; 32] = [
            0x3a, 0x73, 0xf4, 0x12, 0x3a, 0x5c, 0xd2, 0x12, 0x1f, 0x21, 0xcd, 0x7e, 0x8d, 0x35,
            0x88, 0x35, 0x47, 0x69, 0x49, 0xd0, 0x35, 0xd9, 0xc2, 0xda, 0x68, 0x06, 0xb4, 0x63,
            0x3a, 0xc8, 0xc1, 0xe2,
        ];

        // 0xa0434d9e47f3c86235477c7b1ae6ae5d3442d49b1943c2b752a68e2a47e247c7
        let pub_key_x: [u8; 32] = [
            0xa0, 0x43, 0x4d, 0x9e, 0x47, 0xf3, 0xc8, 0x62, 0x35, 0x47, 0x7c, 0x7b, 0x1a, 0xe6,
            0xae, 0x5d, 0x34, 0x42, 0xd4, 0x9b, 0x19, 0x43, 0xc2, 0xb7, 0x52, 0xa6, 0x8e, 0x2a,
            0x47, 0xe2, 0x47, 0xc7,
        ];

        // 0x893aba425419bc27a3b6c7e693a24c696f794c2ed877a1593cbee53b037368d7
        let pub_key_y: [u8; 32] = [
            0x89, 0x3a, 0xba, 0x42, 0x54, 0x19, 0xbc, 0x27, 0xa3, 0xb6, 0xc7, 0xe6, 0x93, 0xa2,
            0x4c, 0x69, 0x6f, 0x79, 0x4c, 0x2e, 0xd8, 0x77, 0xa1, 0x59, 0x3c, 0xbe, 0xe5, 0x3b,
            0x03, 0x73, 0x68, 0xd7,
        ];

        // 0xe5081c80ab427dc370346f4a0e31aa2bad8d9798c38061db9ae55a4e8df454fd28119894344e71b78770cc931d61f480ecbb0b89d6eb69690161e49a715fcd55
        let signature: [u8; 64] = [
            0xe5, 0x08, 0x1c, 0x80, 0xab, 0x42, 0x7d, 0xc3, 0x70, 0x34, 0x6f, 0x4a, 0x0e, 0x31,
            0xaa, 0x2b, 0xad, 0x8d, 0x97, 0x98, 0xc3, 0x80, 0x61, 0xdb, 0x9a, 0xe5, 0x5a, 0x4e,
            0x8d, 0xf4, 0x54, 0xfd, 0x28, 0x11, 0x98, 0x94, 0x34, 0x4e, 0x71, 0xb7, 0x87, 0x70,
            0xcc, 0x93, 0x1d, 0x61, 0xf4, 0x80, 0xec, 0xbb, 0x0b, 0x89, 0xd6, 0xeb, 0x69, 0x69,
            0x01, 0x61, 0xe4, 0x9a, 0x71, 0x5f, 0xcd, 0x55,
        ];

        let valid =
            verify_secp256k1_ecdsa_signature(&hashed_message, &pub_key_x, &pub_key_y, &signature);

        assert!(valid)
    }
}
