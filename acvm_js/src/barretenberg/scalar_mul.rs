use acvm::FieldElement;

use super::{Barretenberg, Error, FIELD_BYTES};

pub(crate) trait ScalarMul {
    fn fixed_base(&self, input: &FieldElement) -> Result<(FieldElement, FieldElement), Error>;
}

impl ScalarMul for Barretenberg {
    fn fixed_base(&self, input: &FieldElement) -> Result<(FieldElement, FieldElement), Error> {
        let lhs_ptr: usize = 0;
        let result_ptr: usize = lhs_ptr + FIELD_BYTES;
        self.transfer_to_heap(&input.to_be_bytes(), lhs_ptr);

        self.call_multiple("compute_public_key", vec![&lhs_ptr.into(), &result_ptr.into()])?;

        let result_bytes: [u8; 2 * FIELD_BYTES] = self.read_memory(result_ptr);
        let (pubkey_x_bytes, pubkey_y_bytes) = result_bytes.split_at(FIELD_BYTES);

        assert!(pubkey_x_bytes.len() == FIELD_BYTES);
        assert!(pubkey_y_bytes.len() == FIELD_BYTES);

        let pubkey_x = FieldElement::from_be_bytes_reduce(pubkey_x_bytes);
        let pubkey_y = FieldElement::from_be_bytes_reduce(pubkey_y_bytes);
        Ok((pubkey_x, pubkey_y))
    }
}
