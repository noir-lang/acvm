use std::io::{Read, Write};

use acir_field::FieldElement;

pub fn read_n<const NUM_BYTES: usize, R: Read>(mut r: R) -> std::io::Result<[u8; NUM_BYTES]> {
    let mut bytes = [0u8; NUM_BYTES];
    r.read_exact(&mut bytes[..])?;
    Ok(bytes)
}
pub fn write_n<const NUM_BYTES: usize, W: Write>(
    w: W,
    bytes: [u8; NUM_BYTES],
) -> std::io::Result<usize> {
    write_bytes(w, &bytes)
}
pub fn write_bytes<W: Write>(mut w: W, bytes: &[u8]) -> std::io::Result<usize> {
    w.write(&bytes)
}

pub fn write_u16<W: Write>(w: W, num: u16) -> std::io::Result<usize> {
    let bytes = num.to_le_bytes();
    write_n::<2, _>(w, bytes)
}
pub fn write_u32<W: Write>(w: W, num: u32) -> std::io::Result<usize> {
    let bytes = num.to_le_bytes();
    write_n::<4, _>(w, bytes)
}

pub fn read_u16<R: Read>(r: R) -> std::io::Result<u16> {
    const NUM_BYTES: usize = 2;
    let bytes = read_n::<NUM_BYTES, _>(r)?;
    Ok(u16::from_le_bytes(bytes))
}
pub fn read_u32<R: Read>(r: R) -> std::io::Result<u32> {
    const NUM_BYTES: usize = 4;
    let bytes = read_n::<NUM_BYTES, _>(r)?;
    Ok(u32::from_le_bytes(bytes))
}
pub fn read_field_element<const NUM_BYTES: usize, R: Read>(
    mut r: R,
) -> std::io::Result<FieldElement> {
    const FIELD_ELEMENT_NUM_BYTES: usize = FieldElement::max_num_bytes() as usize;

    let bytes = read_n::<FIELD_ELEMENT_NUM_BYTES, _>(&mut r)?;

    // TODO: We should not reduce here, we want the serialisation to be
    // TODO canonical
    let field_element = FieldElement::from_be_bytes_reduce(&bytes);

    Ok(field_element)
}
