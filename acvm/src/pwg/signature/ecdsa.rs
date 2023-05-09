use acir::{circuit::opcodes::FunctionInput, native_types::Witness, FieldElement};
use std::collections::BTreeMap;

use crate::{pwg::witness_to_value, pwg::OpcodeResolution, OpcodeResolutionError};

fn to_u8_vec(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    value: &[FunctionInput],
) -> Result<Vec<u8>, OpcodeResolutionError> {
    let mut result = Vec::new();
    for input in value {
        let w_value = witness_to_value(initial_witness, input.witness)?.to_be_bytes();
        let byte = w_value.last().unwrap();
        result.push(*byte);
    }
    Ok(result)
}

pub fn secp256k1_prehashed(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    public_key_x_inputs: &[FunctionInput],
    public_key_y_inputs: &[FunctionInput],
    signature_inputs: &[FunctionInput],
    message_inputs: &[FunctionInput],
    output: Witness,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let pub_key_x: [u8; 32] =
        to_u8_vec(initial_witness, public_key_x_inputs)?.try_into().unwrap_or_else(|_| {
            panic!("pub_key_x should be 32 bytes long, found {} bytes", public_key_x_inputs.len())
        });
    let pub_key_y: [u8; 32] =
        to_u8_vec(initial_witness, public_key_y_inputs)?.try_into().unwrap_or_else(|_| {
            panic!("pub_key_y should be 32 bytes long, found {} bytes", public_key_y_inputs.len())
        });
    let signature: [u8; 32] =
        to_u8_vec(initial_witness, signature_inputs)?.try_into().unwrap_or_else(|_| {
            panic!("signature should be 64 bytes long, found {} bytes", signature_inputs.len())
        });

    let mut hashed_message = Vec::new();
    for msg in message_inputs.iter() {
        let msg_i_field = witness_to_value(initial_witness, msg.witness)?;
        let msg_i = *msg_i_field.to_be_bytes().last().unwrap();
        hashed_message.push(msg_i);
    }

    let result =
        ecdsa_secp256k1::verify_prehashed(&hashed_message, &pub_key_x, &pub_key_y, &signature)
            .is_ok();

    initial_witness.insert(output, FieldElement::from(result));
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
}
