use acvm::FieldElement;

use super::{Assignments, Barretenberg, Error, FIELD_BYTES};

pub(crate) trait Pedersen {
    fn encrypt(&self, inputs: Vec<FieldElement>) -> Result<(FieldElement, FieldElement), Error>;
}

impl Pedersen for Barretenberg {
    fn encrypt(&self, inputs: Vec<FieldElement>) -> Result<(FieldElement, FieldElement), Error> {
        let input_buf = Assignments::from(inputs).to_bytes();
        let input_ptr = self.allocate(&input_buf)?;
        let result_ptr: usize = 0;

        self.call_multiple("pedersen_plookup_commit", vec![&input_ptr, &result_ptr.into()])?;

        let result_bytes: [u8; 2 * FIELD_BYTES] = self.read_memory(result_ptr);
        let (point_x_bytes, point_y_bytes) = result_bytes.split_at(FIELD_BYTES);

        let point_x = FieldElement::from_be_bytes_reduce(point_x_bytes);
        let point_y = FieldElement::from_be_bytes_reduce(point_y_bytes);

        Ok((point_x, point_y))
    }
}
