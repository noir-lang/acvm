use std::io::{Read, Write};

use crate::{
    native_types::{Expression, Witness},
    serialization::{read_n, read_u16, read_u32, write_bytes, write_u16, write_u32},
};
use acir_field::FieldElement;
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
        a: Expression,
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

    //decomposition of a: a=\sum b[i]*radix^i where b is an array of witnesses < radix in little endian form
    ToLeRadix {
        a: Expression,
        b: Vec<Witness>,
        radix: u32,
    },

    // Sort directive, using a sorting network
    // This directive is used to generate the values of the control bits for the sorting network such that its outputs are properly sorted according to sort_by
    PermutationSort {
        inputs: Vec<Vec<Expression>>, // Array of tuples to sort
        tuple: u32, // tuple size; if 1 then inputs is a single array [a0,a1,..], if 2 then inputs=[(a0,b0),..] is [a0,b0,a1,b1,..], etc..
        bits: Vec<Witness>, // control bits of the network which permutes the inputs into its sorted version
        sort_by: Vec<u32>, // specify primary index to sort by, then the secondary,... For instance, if tuple is 2 and sort_by is [1,0], then a=[(a0,b0),..] is sorted by bi and then ai.
    },

    Log {
        trace_label: Option<String>, // This field states whether the log should be further manipulated or simply displayed to standard output. It is simply passed along for the caller of the PWG to implement
        output_info: LogOutputInfo,
    },
}

impl Directive {
    pub fn name(&self) -> &str {
        match self {
            Directive::Invert { .. } => "invert",
            Directive::Quotient { .. } => "quotient",
            Directive::Truncate { .. } => "truncate",
            Directive::OddRange { .. } => "odd_range",
            Directive::ToLeRadix { .. } => "to_le_radix",
            Directive::PermutationSort { .. } => "permutation_sort",
            Directive::Log { .. } => "log",
        }
    }
    fn to_u16(&self) -> u16 {
        match self {
            Directive::Invert { .. } => 0,
            Directive::Quotient { .. } => 1,
            Directive::Truncate { .. } => 2,
            Directive::OddRange { .. } => 3,
            Directive::ToLeRadix { .. } => 4,
            Directive::Log { .. } => 5,
            Directive::PermutationSort { .. } => 6,
        }
    }

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        write_u16(&mut writer, self.to_u16())?;
        match self {
            Directive::Invert { x, result } => {
                write_u32(&mut writer, x.witness_index())?;
                write_u32(&mut writer, result.witness_index())?;
            }
            Directive::Quotient { a, b, q, r, predicate } => {
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
                a.write(&mut writer)?;
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
            Directive::ToLeRadix { a, b, radix } => {
                a.write(&mut writer)?;
                write_u32(&mut writer, b.len() as u32)?;
                for bit in b {
                    write_u32(&mut writer, bit.witness_index())?;
                }
                write_u32(&mut writer, *radix)?;
            }
            Directive::PermutationSort { inputs: a, tuple, bits, sort_by } => {
                write_u32(&mut writer, *tuple)?;
                write_u32(&mut writer, a.len() as u32)?;
                for e in a {
                    for i in 0..*tuple {
                        e[i as usize].write(&mut writer)?;
                    }
                }
                write_u32(&mut writer, bits.len() as u32)?;
                for b in bits {
                    write_u32(&mut writer, b.witness_index())?;
                }
                write_u32(&mut writer, sort_by.len() as u32)?;
                for i in sort_by {
                    write_u32(&mut writer, *i)?;
                }
            }
            Directive::Log { trace_label, output_info } => {
                if let Some(trace_label) = trace_label {
                    write_bytes(&mut writer, trace_label.as_bytes())?;
                }
                match output_info {
                    LogOutputInfo::FinalizedOutput(output_string) => {
                        write_bytes(&mut writer, output_string.as_bytes())?;
                    }
                    LogOutputInfo::WitnessOutput(witnesses) => {
                        write_u32(&mut writer, witnesses.len() as u32)?;
                        for w in witnesses {
                            write_u32(&mut writer, w.witness_index())?;
                        }
                    }
                }
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

                Ok(Directive::Quotient { a, b, q, r, predicate })
            }
            2 => {
                let a = Expression::read(&mut reader)?;
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

                let radix = read_u32(&mut reader)?;

                Ok(Directive::ToLeRadix { a, b, radix })
            }
            6 => {
                let tuple = read_u32(&mut reader)?;
                let a_len = read_u32(&mut reader)?;
                let mut a = Vec::with_capacity(a_len as usize);
                for _ in 0..a_len {
                    let mut element = Vec::new();
                    for _ in 0..tuple {
                        element.push(Expression::read(&mut reader)?);
                    }
                    a.push(element);
                }

                let bits_len = read_u32(&mut reader)?;
                let mut bits = Vec::with_capacity(bits_len as usize);
                for _ in 0..bits_len {
                    bits.push(Witness(read_u32(&mut reader)?));
                }
                let sort_by_len = read_u32(&mut reader)?;
                let mut sort_by = Vec::with_capacity(sort_by_len as usize);
                for _ in 0..sort_by_len {
                    sort_by.push(read_u32(&mut reader)?);
                }
                Ok(Directive::PermutationSort { inputs: a, tuple, bits, sort_by })
            }

            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// This info is used when solving the initial witness
/// If values are compile time and/or known during
/// evaluation, we can form an output string during ACIR generation.
/// Otherwise, we must store witnesses whose values will
/// be fetched during the PWG stage.
pub enum LogOutputInfo {
    FinalizedOutput(String),
    WitnessOutput(Vec<Witness>),
}

pub struct SolvedLog {
    pub trace_label: Option<String>,
    pub output_info: SolvedLogOutputInfo,
}

pub enum SolvedLogOutputInfo {
    FinalizedOutput(String),
    WitnessValues(Vec<FieldElement>),
}

#[test]
fn serialization_roundtrip() {
    fn read_write(directive: Directive) -> (Directive, Directive) {
        let mut bytes = Vec::new();
        directive.write(&mut bytes).unwrap();
        let got_dir = Directive::read(&*bytes).unwrap();

        (directive, got_dir)
    }
    // TODO: Find a way to ensure that we include all of the variants
    let invert = Directive::Invert { x: Witness(10), result: Witness(10) };

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
        a: Expression::default(),
        b: Witness(2u32),
        c: Witness(3u32),
        bit_size: 123,
    };

    let odd_range =
        Directive::OddRange { a: Witness(1u32), b: Witness(2u32), r: Witness(3u32), bit_size: 32 };

    let to_le_radix = Directive::ToLeRadix {
        a: Expression::default(),
        b: vec![Witness(1u32), Witness(2u32), Witness(3u32), Witness(4u32)],
        radix: 4,
    };

    let directives =
        vec![invert, quotient_none, quotient_predicate, truncate, odd_range, to_le_radix];

    for directive in directives {
        let (dir, got_dir) = read_write(directive);
        assert_eq!(dir, got_dir);
    }
}
