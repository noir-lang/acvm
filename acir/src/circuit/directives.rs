use crate::native_types::{Expression, Witness};
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
        predicate: Option<Box<Expression>>,
    },

    //Reduces the value of a modulo 2^bit_size and stores the result in b: a= c*2^bit_size + b
    Truncate {
        a: Witness,
        b: Witness,
        c: Witness,
        bit_size: u32,
    },

    //Computes the highest bit b of a: a = b*2^(bit_size-1) + r, where a<2^bit_size, b is 0 or 1 and r<2^(bit_size-1)
    Oddrange {
        a: Witness,
        b: Witness,
        r: Witness,
        bit_size: u32,
    },

    //Bit decomposition of a: a=\sum b[i]*2^i
    Split {
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
