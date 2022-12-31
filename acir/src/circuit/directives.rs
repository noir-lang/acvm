use std::io::{Read, Write};

use crate::{
    native_types::{Expression, Witness},
    serialisation::{read_n, read_u16, read_u32, write_bytes, write_u16, write_u32},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Directives do not apply any constraints.
/// You can think of them as opcodes that allow one to use non-determinism
/// In the future, this can be replaced with asm non-determinism blocks
pub enum Directive {
    //Inverts the value of x and stores it in the result variable
    Invert {
        x: Witness,
        result: Witness,
    },

    //Performs euclidian division of a / b (as integers) and stores the quotient in q and the rest in r
    Quotient {
        a: Expression,
        b: Expression,
        q: Witness,
        r: Witness,
        predicate: Option<Expression>,
    },

    //Reduces the value of a modulo 2^bit_size and stores the result in b: a= c*2^bit_size + b
    Truncate {
        a: Witness,
        b: Witness,
        c: Witness,
        bit_size: u32,
    },

    //Computes the highest bit b of a: a = b*2^(bit_size-1) + r, where a<2^bit_size, b is 0 or 1 and r<2^(bit_size-1)
    OddRange {
        a: Witness,
        b: Witness,
        r: Witness,
        bit_size: u32,
    },

    //Bit decomposition of a: a=\sum b[i]*2^i
    ToBits {
        a: Expression,
        b: Vec<Witness>,
        bit_size: u32,
    },

    //Byte decomposition of a: a=\sum b[i]*2^i where b is a byte array
    ToBytes {
        a: Expression,
        b: Vec<Witness>,
        byte_size: u32,
    },
}

impl Directive {
    pub fn name(&self) -> &str {
        match self {
            Directive::Invert { .. } => "invert",
            Directive::Quotient { .. } => "quotient",
            Directive::Truncate { .. } => "truncate",
            Directive::OddRange { .. } => "odd_range",
            Directive::ToBits { .. } => "to_bits",
            Directive::ToBytes { .. } => "to_bytes",
        }
    }
    fn to_u16(&self) -> u16 {
        match self {
            Directive::Invert { .. } => 0,
            Directive::Quotient { .. } => 1,
            Directive::Truncate { .. } => 2,
            Directive::OddRange { .. } => 3,
            Directive::ToBits { .. } => 4,
            Directive::ToBytes { .. } => 5,
        }
    }

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        write_u16(&mut writer, self.to_u16())?;
        match self {
            Directive::Invert { x, result } => {
                write_u32(&mut writer, x.witness_index())?;
                write_u32(&mut writer, result.witness_index())?;
            }
            Directive::Quotient {
                a,
                b,
                q,
                r,
                predicate,
            } => {
                a.write(&mut writer)?;
                b.write(&mut writer)?;
                write_u32(&mut writer, q.witness_index())?;
                write_u32(&mut writer, r.witness_index())?;

                let predicate_is_some = vec![predicate.is_some() as u8];
                write_bytes(&mut writer, &predicate_is_some)?;

                if let Some(pred) = predicate {
                    pred.write(&mut writer)?;
                }
            }
            Directive::Truncate { a, b, c, bit_size } => {
                write_u32(&mut writer, a.witness_index())?;
                write_u32(&mut writer, b.witness_index())?;
                write_u32(&mut writer, c.witness_index())?;
                write_u32(&mut writer, *bit_size)?;
            }
            Directive::OddRange { a, b, r, bit_size } => {
                write_u32(&mut writer, a.witness_index())?;
                write_u32(&mut writer, b.witness_index())?;
                write_u32(&mut writer, r.witness_index())?;
                write_u32(&mut writer, *bit_size)?;
            }
            Directive::ToBits { a, b, bit_size } => {
                a.write(&mut writer)?;

                // The length of the bit vector is the same as the bit_size
                // TODO: can we omit the bit_size altogether then?
                write_u32(&mut writer, b.len() as u32)?;
                for bit in b {
                    write_u32(&mut writer, bit.witness_index())?;
                }

                write_u32(&mut writer, *bit_size)?;
            }
            Directive::ToBytes { a, b, byte_size } => {
                a.write(&mut writer)?;

                // TODO: can we omit the byte_size altogether?
                // TODO see comment on ToBits about inferring this
                // TODO from the size of the vector
                write_u32(&mut writer, b.len() as u32)?;
                for bit in b {
                    write_u32(&mut writer, bit.witness_index())?;
                }

                write_u32(&mut writer, *byte_size)?;
            }
        };

        Ok(())
    }

    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let directive_index = read_u16(&mut reader)?;

        match directive_index {
            0 => {
                let x = Witness(read_u32(&mut reader)?);
                let result = Witness(read_u32(&mut reader)?);
                Ok(Directive::Invert { x, result })
            }
            1 => {
                let a = Expression::read(&mut reader)?;
                let b = Expression::read(&mut reader)?;
                let q = Witness(read_u32(&mut reader)?);
                let r = Witness(read_u32(&mut reader)?);

                // Read byte to figure out if there is a predicate
                let predicate_is_some = read_n::<1, _>(&mut reader)?[0] != 0;
                let predicate = match predicate_is_some {
                    true => Some(Expression::read(&mut reader)?),
                    false => None,
                };

                Ok(Directive::Quotient {
                    a,
                    b,
                    q,
                    r,
                    predicate,
                })
            }
            2 => {
                let a = Witness(read_u32(&mut reader)?);
                let b = Witness(read_u32(&mut reader)?);
                let c = Witness(read_u32(&mut reader)?);
                let bit_size = read_u32(&mut reader)?;
                Ok(Directive::Truncate { a, b, c, bit_size })
            }
            3 => {
                let a = Witness(read_u32(&mut reader)?);
                let b = Witness(read_u32(&mut reader)?);
                let r = Witness(read_u32(&mut reader)?);
                let bit_size = read_u32(&mut reader)?;
                Ok(Directive::OddRange { a, b, r, bit_size })
            }
            4 => {
                let a = Expression::read(&mut reader)?;
                let b_len = read_u32(&mut reader)?;
                let mut b = Vec::with_capacity(b_len as usize);
                for _ in 0..b_len {
                    let witness = Witness(read_u32(&mut reader)?);
                    b.push(witness)
                }

                let bit_size = read_u32(&mut reader)?;

                Ok(Directive::ToBits { a, b, bit_size })
            }
            5 => {
                let a = Expression::read(&mut reader)?;
                let b_len = read_u32(&mut reader)?;
                let mut b = Vec::with_capacity(b_len as usize);
                for _ in 0..b_len {
                    let witness = Witness(read_u32(&mut reader)?);
                    b.push(witness)
                }

                let byte_size = read_u32(&mut reader)?;

                Ok(Directive::ToBytes { a, b, byte_size })
            }
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }
}

#[test]
fn serialisation_roundtrip() {
    fn read_write(directive: Directive) -> (Directive, Directive) {
        let mut bytes = Vec::new();
        directive.write(&mut bytes).unwrap();
        let got_dir = Directive::read(&*bytes).unwrap();

        (directive, got_dir)
    }
    // TODO: Find a way to ensure that we include all of the variants
    let invert = Directive::Invert {
        x: Witness(10),
        result: Witness(10),
    };

    let quotient_none = Directive::Quotient {
        a: Expression::default(),
        b: Expression::default(),
        q: Witness(1u32),
        r: Witness(2u32),
        predicate: None,
    };
    let quotient_predicate = Directive::Quotient {
        a: Expression::default(),
        b: Expression::default(),
        q: Witness(1u32),
        r: Witness(2u32),
        predicate: Some(Expression::default()),
    };

    let truncate = Directive::Truncate {
        a: Witness(1u32),
        b: Witness(2u32),
        c: Witness(3u32),
        bit_size: 123,
    };

    let odd_range = Directive::OddRange {
        a: Witness(1u32),
        b: Witness(2u32),
        r: Witness(3u32),
        bit_size: 32,
    };

    let to_bits = Directive::ToBits {
        a: Expression::default(),
        b: vec![Witness(1u32), Witness(2u32)],
        bit_size: 2,
    };
    let to_bytes = Directive::ToBytes {
        a: Expression::default(),
        b: vec![Witness(1u32), Witness(2u32), Witness(3u32), Witness(4u32)],
        byte_size: 4,
    };

    let directives = vec![
        invert,
        quotient_none,
        quotient_predicate,
        truncate,
        odd_range,
        to_bits,
        to_bytes,
    ];

    for directive in directives {
        let (dir, got_dir) = read_write(directive);
        assert_eq!(dir, got_dir);
    }
}
