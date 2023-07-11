use acir_field::FieldElement;
use brillig::{BinaryFieldOp, BinaryIntOp};

/// Evaluate a binary operation on two FieldElements and return the result as a FieldElement.
pub(crate) fn evaluate_binary_field_op(
    op: &BinaryFieldOp,
    a: FieldElement,
    b: FieldElement,
) -> FieldElement {
    match op {
        // Perform addition, subtraction, multiplication, and division based on the BinaryOp variant.
        BinaryFieldOp::Add => a + b,
        BinaryFieldOp::Sub => a - b,
        BinaryFieldOp::Mul => a * b,
        BinaryFieldOp::Div => a / b,
        BinaryFieldOp::Equals => (a == b).into(),
    }
}

/// Evaluate a binary operation on two unsigned integers (u128) with a given bit size and return the result as a u128.
pub(crate) fn evaluate_binary_int_op(op: &BinaryIntOp, a: u128, b: u128, bit_size: u32) -> u128 {
    let bit_modulo = 1_u128 << bit_size;
    match op {
        // Perform addition, subtraction, and multiplication, applying a modulo operation to keep the result within the bit size.
        BinaryIntOp::Add => a.wrapping_add(b) % bit_modulo,
        BinaryIntOp::Sub => a.wrapping_sub(b) % bit_modulo,
        BinaryIntOp::Mul => a.wrapping_mul(b) % bit_modulo,
        // Perform unsigned division using the modulo operation on a and b.
        BinaryIntOp::UnsignedDiv => (a % bit_modulo) / (b % bit_modulo),
        // Perform signed division by first converting a and b to signed integers and then back to unsigned after the operation.
        BinaryIntOp::SignedDiv => {
            to_unsigned(to_signed(a, bit_size) / to_signed(b, bit_size), bit_size)
        }
        // Perform a == operation, returning 0 or 1
        BinaryIntOp::Equals => ((a % bit_modulo) == (b % bit_modulo)).into(),
        // Perform a < operation, returning 0 or 1
        BinaryIntOp::LessThan => ((a % bit_modulo) < (b % bit_modulo)).into(),
        // Perform a <= operation, returning 0 or 1
        BinaryIntOp::LessThanEquals => ((a % bit_modulo) <= (b % bit_modulo)).into(),
        // Perform bitwise AND, OR, XOR, left shift, and right shift operations, applying a modulo operation to keep the result within the bit size.
        BinaryIntOp::And => (a & b) % bit_modulo,
        BinaryIntOp::Or => (a | b) % bit_modulo,
        BinaryIntOp::Xor => (a ^ b) % bit_modulo,
        BinaryIntOp::Shl => (a << b) % bit_modulo,
        BinaryIntOp::Shr => (a >> b) % bit_modulo,
    }
}

fn to_signed(a: u128, bit_size: u32) -> i128 {
    assert!(bit_size < 128);
    let pow_2 = 2_u128.pow(bit_size - 1);
    if a < pow_2 {
        a as i128
    } else {
        (a.wrapping_sub(2 * pow_2)) as i128
    }
}

fn to_unsigned(a: i128, bit_size: u32) -> u128 {
    if a >= 0 {
        a as u128
    } else {
        (a + 2_i128.pow(bit_size)) as u128
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestParams {
        a: u128,
        b: u128,
        result: u128,
    }

    fn to_negative(a: u128, bit_size: u32) -> u128 {
        assert!(a > 0);
        let two_pow = 2_u128.pow(bit_size);
        two_pow - a
    }

    fn evaluate_int_ops(test_params: Vec<TestParams>, op: BinaryIntOp, bit_size: u32) {
        for test in test_params {
            assert_eq!(evaluate_binary_int_op(&op, test.a, test.b, bit_size), test.result);
        }
    }

    #[test]
    fn add_test() {
        let bit_size = 4;

        let test_ops = vec![
            TestParams { a: 5, b: 10, result: 15 },
            TestParams { a: 10, b: 10, result: 4 },
            TestParams { a: 5, b: to_negative(3, bit_size), result: 2 },
            TestParams { a: to_negative(3, bit_size), b: 1, result: to_negative(2, bit_size) },
            TestParams { a: 5, b: to_negative(6, bit_size), result: to_negative(1, bit_size) },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Add, bit_size);
    }

    #[test]
    fn sub_test() {
        let bit_size = 4;

        let test_ops = vec![
            TestParams { a: 5, b: 3, result: 2 },
            TestParams { a: 5, b: 10, result: to_negative(5, bit_size) },
            TestParams { a: 5, b: to_negative(3, bit_size), result: 8 },
            TestParams { a: to_negative(3, bit_size), b: 2, result: to_negative(5, bit_size) },
            TestParams { a: 14, b: to_negative(3, bit_size), result: 1 },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Sub, bit_size);
    }

    #[test]
    fn mul_test() {
        let bit_size = 4;

        let test_ops = vec![
            TestParams { a: 5, b: 3, result: 15 },
            TestParams { a: 5, b: 10, result: 2 },
            TestParams { a: to_negative(1, bit_size), b: to_negative(5, bit_size), result: 5 },
            TestParams { a: to_negative(1, bit_size), b: 5, result: to_negative(5, bit_size) },
            TestParams {
                a: to_negative(2, bit_size),
                b: 7,
                // negative 14 wraps to a 2
                result: to_negative(14, bit_size),
            },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::Mul, bit_size);

        let bit_size = 127;
        let a = 2_u128.pow(bit_size) - 1;
        let b = 3;

        // ( 2**(n-1) - 1 ) * 3 = 2*2**(n-1) - 2 + (2**(n-1) - 1) => wraps to (2**(n-1) - 1) - 2
        assert_eq!(evaluate_binary_int_op(&BinaryIntOp::Mul, a, b, bit_size), a - 2);
    }

    #[test]
    fn div_test() {
        let bit_size = 4;

        let test_ops =
            vec![TestParams { a: 5, b: 3, result: 1 }, TestParams { a: 5, b: 10, result: 0 }];

        evaluate_int_ops(test_ops, BinaryIntOp::UnsignedDiv, bit_size);
    }

    #[test]
    fn to_signed_roundtrip() {
        let bit_size = 32;
        let minus_one = 2_u128.pow(bit_size) - 1;
        assert_eq!(to_unsigned(to_signed(minus_one, bit_size), bit_size), minus_one);
    }

    #[test]
    fn signed_div_test() {
        let bit_size = 32;

        let test_ops = vec![
            TestParams { a: 5, b: to_negative(10, bit_size), result: 0 },
            TestParams { a: 5, b: to_negative(1, bit_size), result: to_negative(5, bit_size) },
            TestParams { a: to_negative(5, bit_size), b: to_negative(1, bit_size), result: 5 },
        ];

        evaluate_int_ops(test_ops, BinaryIntOp::SignedDiv, bit_size);
    }
}
