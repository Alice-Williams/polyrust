//! Independent expected tables for the measured LP64 scalar model.
use crate::ast::CScalarType;
use CScalarType::{Bool, F64, I8, I16, I32, I64, Int, PlainChar, Size, U8, U16, U32, U64};

#[test]
fn every_scalar_pair_has_the_exact_measured_arithmetic_result() {
    const SMALL: [CScalarType; 13] = [
        Int, Int, Int, Int, Int, Int, Int, Int, U32, I64, U64, U64, F64,
    ];
    const UNSIGNED: [CScalarType; 13] = [
        U32, U32, U32, U32, U32, U32, U32, U32, U32, I64, U64, U64, F64,
    ];
    const LONG: [CScalarType; 13] = [
        I64, I64, I64, I64, I64, I64, I64, I64, I64, I64, U64, U64, F64,
    ];
    const ULONG: [CScalarType; 13] = [
        U64, U64, U64, U64, U64, U64, U64, U64, U64, U64, U64, U64, F64,
    ];
    const TABLE: [[CScalarType; 13]; 13] = [
        SMALL, SMALL, SMALL, SMALL, SMALL, SMALL, SMALL, SMALL, UNSIGNED, LONG, ULONG, ULONG,
        [F64; 13],
    ];
    for (row, left) in CScalarType::ALL.into_iter().enumerate() {
        for (column, right) in CScalarType::ALL.into_iter().enumerate() {
            assert_eq!(
                left.usual_arithmetic_conversion(right),
                TABLE[row][column],
                "{left:?}/{right:?}"
            );
        }
    }
}

#[test]
fn every_promotion_and_compatible_alias_is_explicit() {
    let promotions = [
        (Bool, Some(Int)),
        (PlainChar, Some(Int)),
        (Int, Some(Int)),
        (I8, Some(Int)),
        (U8, Some(Int)),
        (I16, Some(Int)),
        (U16, Some(Int)),
        (I32, Some(Int)),
        (U32, Some(U32)),
        (I64, Some(I64)),
        (U64, Some(U64)),
        (Size, Some(U64)),
        (F64, None),
    ];
    for (scalar, expected) in promotions {
        assert_eq!(scalar.integer_promotion(), expected);
    }
    for left in CScalarType::ALL {
        for right in CScalarType::ALL {
            let expected = left == right
                || matches!(
                    (left, right),
                    (Int, I32) | (I32, Int) | (U64, Size) | (Size, U64)
                );
            assert_eq!(
                left.is_compatible_with(right),
                expected,
                "{left:?}/{right:?}"
            );
        }
    }
    assert_ne!(
        I32, Int,
        "AST spellings retain distinct declared provenance"
    );
    assert_ne!(PlainChar, I8, "equal layout is not C type compatibility");
}
