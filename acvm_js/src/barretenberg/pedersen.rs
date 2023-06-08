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

#[test]
fn pedersen_hash_to_point() -> Result<(), Error> {
    let barretenberg = Barretenberg::new();
    let (x, y) = barretenberg.encrypt(vec![FieldElement::zero(), FieldElement::one()])?;
    let expected_x = FieldElement::from_hex(
        "0x11831f49876c313f2a9ec6d8d521c7ce0b6311c852117e340bfe27fd1ac096ef",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x0ecf9d98be4597a88c46a7e0fa8836b57a7dcb41ee30f8d8787b11cc259c83fa",
    )
    .unwrap();

    assert_eq!(expected_x.to_hex(), x.to_hex());
    assert_eq!(expected_y.to_hex(), y.to_hex());
    Ok(())
}
// }
