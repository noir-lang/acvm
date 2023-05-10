use acir::{circuit::opcodes::FunctionInput, native_types::Witness, FieldElement};
use std::collections::BTreeMap;

use crate::{pwg::witness_to_value, pwg::OpcodeResolution, OpcodeResolutionError};

pub fn secp256k1_prehashed(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let mut inputs_iter = inputs.iter();

    let mut pub_key_x = [0u8; 32];
    for (i, pkx) in pub_key_x.iter_mut().enumerate() {
        let _x_i = inputs_iter
            .next()
            .unwrap_or_else(|| panic!("pub_key_x should be 32 bytes long, found only {i} bytes"));

        let x_i = witness_to_value(initial_witness, _x_i.witness)?;
        *pkx = *x_i.to_be_bytes().last().unwrap();
    }

    let mut pub_key_y = [0u8; 32];
    for (i, pky) in pub_key_y.iter_mut().enumerate() {
        let _y_i = inputs_iter
            .next()
            .unwrap_or_else(|| panic!("pub_key_y should be 32 bytes long, found only {i} bytes"));

        let y_i = witness_to_value(initial_witness, _y_i.witness)?;
        *pky = *y_i.to_be_bytes().last().unwrap();
    }

    let mut signature = [0u8; 64];
    for (i, sig) in signature.iter_mut().enumerate() {
        let _sig_i = inputs_iter
            .next()
            .unwrap_or_else(|| panic!("signature should be 64 bytes long, found only {i} bytes"));

        let sig_i = witness_to_value(initial_witness, _sig_i.witness)?;
        *sig = *sig_i.to_be_bytes().last().unwrap()
    }

    let mut hashed_message = Vec::new();
    for msg in inputs_iter {
        let msg_i_field = witness_to_value(initial_witness, msg.witness)?;
        let msg_i = *msg_i_field.to_be_bytes().last().unwrap();
        hashed_message.push(msg_i);
    }

    let result =
        ecdsa_secp256k1::verify_prehashed(&hashed_message, &pub_key_x, &pub_key_y, &signature)
            .is_ok();

    initial_witness.insert(outputs[0], FieldElement::from(result));
    Ok(OpcodeResolution::Solved)
}

mod ecdsa_secp256k1 {
    use std::convert::TryInto;

    use k256::{ecdsa::Signature, Scalar};
    use k256::{
        elliptic_curve::sec1::{Coordinates, ToEncodedPoint},
        AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
    };
    // This method is used to generate test vectors
    // in noir. TODO: check that it is indeed used
    #[allow(dead_code)]
    fn generate_proof_data() {
        use k256::ecdsa::{signature::Signer, SigningKey};

        use sha2::{Digest, Sha256};

        // Signing
        let signing_key = SigningKey::from_bytes(&[2u8; 32]).unwrap();
        let message =
            b"ECDSA proves knowledge of a secret number in the context of a single message";

        let mut hasher = Sha256::new();
        hasher.update(message);
        let digest = hasher.finalize();

        let signature: Signature = signing_key.sign(message);
        // Verification
        use k256::ecdsa::{signature::Verifier, VerifyingKey};

        let verify_key = VerifyingKey::from(&signing_key);

        if let Coordinates::Uncompressed { x, y } = verify_key.to_encoded_point(false).coordinates()
        {
            let signature_bytes: &[u8] = signature.as_ref();
            assert!(Signature::try_from(signature_bytes).unwrap() == signature);
            verify_prehashed(&digest, x, y, signature_bytes).unwrap();
        } else {
            unreachable!();
        }

        assert!(verify_key.verify(message, &signature).is_ok());
    }

    /// Verify an ECDSA signature, given the hashed message
    pub(super) fn verify_prehashed(
        hashed_msg: &[u8],
        public_key_x_bytes: &[u8],
        public_key_y_bytes: &[u8],
        signature: &[u8],
    ) -> Result<(), ()> {
        // Convert the inputs into k256 data structures

        let signature = Signature::try_from(signature).unwrap();

        let pub_key_x_arr: [u8; 32] = {
            let pub_key_x_bytes: &[u8] = public_key_x_bytes;
            pub_key_x_bytes.try_into().unwrap()
        };
        let pub_key_y_arr: [u8; 32] = {
            let pub_key_y_bytes: &[u8] = public_key_y_bytes;
            pub_key_y_bytes.try_into().unwrap()
        };

        let point = EncodedPoint::from_affine_coordinates(
            &pub_key_x_arr.into(),
            &pub_key_y_arr.into(),
            true,
        );
        let pubkey = PublicKey::try_from(point).unwrap();

        let z = Scalar::from_bytes_reduced(hashed_msg.into());

        // Finished converting bytes into data structures

        let r = signature.r();
        let s = signature.s();

        // Ensure signature is "low S" normalized ala BIP 0062
        if s.is_high().into() {
            return Err(());
        }

        let s_inv = s.invert().unwrap();
        let u1 = z * s_inv;
        let u2 = *r * s_inv;

        #[allow(non_snake_case)]
        let R: AffinePoint = ((ProjectivePoint::generator() * u1)
            + (ProjectivePoint::from(*pubkey.as_affine()) * u2))
            .to_affine();

        if let Coordinates::Uncompressed { x, y: _ } = R.to_encoded_point(false).coordinates() {
            if Scalar::from_bytes_reduced(x).eq(&r) {
                return Ok(());
            }
        }
        Err(())
    }

    #[test]
    fn smoke() {
        let hashed_message: [u8; 32] = [
            0x3a, 0x73, 0xf4, 0x12, 0x3a, 0x5c, 0xd2, 0x12, 0x1f, 0x21, 0xcd, 0x7e, 0x8d, 0x35,
            0x88, 0x35, 0x47, 0x69, 0x49, 0xd0, 0x35, 0xd9, 0xc2, 0xda, 0x68, 0x06, 0xb4, 0x63,
            0x3a, 0xc8, 0xc1, 0xe2,
        ];

        let pub_key_x: [u8; 32] = [
            0xa0, 0x43, 0x4d, 0x9e, 0x47, 0xf3, 0xc8, 0x62, 0x35, 0x47, 0x7c, 0x7b, 0x1a, 0xe6,
            0xae, 0x5d, 0x34, 0x42, 0xd4, 0x9b, 0x19, 0x43, 0xc2, 0xb7, 0x52, 0xa6, 0x8e, 0x2a,
            0x47, 0xe2, 0x47, 0xc7,
        ];
        let pub_key_y: [u8; 32] = [
            0x89, 0x3a, 0xba, 0x42, 0x54, 0x19, 0xbc, 0x27, 0xa3, 0xb6, 0xc7, 0xe6, 0x93, 0xa2,
            0x4c, 0x69, 0x6f, 0x79, 0x4c, 0x2e, 0xd8, 0x77, 0xa1, 0x59, 0x3c, 0xbe, 0xe5, 0x3b,
            0x03, 0x73, 0x68, 0xd7,
        ];
        let signature: [u8; 64] = [
            0xe5, 0x08, 0x1c, 0x80, 0xab, 0x42, 0x7d, 0xc3, 0x70, 0x34, 0x6f, 0x4a, 0x0e, 0x31,
            0xaa, 0x2b, 0xad, 0x8d, 0x97, 0x98, 0xc3, 0x80, 0x61, 0xdb, 0x9a, 0xe5, 0x5a, 0x4e,
            0x8d, 0xf4, 0x54, 0xfd, 0x28, 0x11, 0x98, 0x94, 0x34, 0x4e, 0x71, 0xb7, 0x87, 0x70,
            0xcc, 0x93, 0x1d, 0x61, 0xf4, 0x80, 0xec, 0xbb, 0x0b, 0x89, 0xd6, 0xeb, 0x69, 0x69,
            0x01, 0x61, 0xe4, 0x9a, 0x71, 0x5f, 0xcd, 0x55,
        ];

        verify_prehashed(&hashed_message, &pub_key_x, &pub_key_y, &signature).unwrap();
    }
}
