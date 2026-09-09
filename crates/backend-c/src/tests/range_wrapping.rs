//! A valid Size result is not interchangeable with nonwrapping extent evidence.
use super::integer_tests::{interval, number};
use super::{CScalarRange as Range, CTransfer};
use crate::ast::{CBinaryOperator as B, CScalarType as T, CUnaryOperator as U};
use crate::ownership::{CSafetyError as E, constants::limits};

#[test]
fn size_boundary_arithmetic_distinguishes_representable_progress_from_modulo_results() {
    let max = i128::from(u64::MAX);
    let cases = [
        (B::Add, max - 1, 1, max, false),
        (B::Add, max, 1, 0, true),
        (B::Subtract, 0, 1, max, true),
        (B::Multiply, max, max, 1, true),
        (B::Multiply, max / 2, 2, max - 1, false),
        (B::ShiftLeft, 1, 63, 1_i128 << 63, false),
        (B::ShiftLeft, 2, 63, 0, true),
    ];
    for (op, a, b, expected, wraps) in cases {
        let transfer = Range::exact(number(T::Size, a))
            .binary(op, Range::exact(number(T::Size, b)))
            .unwrap();
        assert_eq!(
            matches!(transfer, CTransfer::MayWrap(_)),
            wraps,
            "{a} {op:?} {b}"
        );
        // size_t shares U64's arithmetic rank on the frozen ABI. Assignment
        // back to a Size slot is a compatible, nonwrapping normalization.
        assert_eq!(transfer.range(), interval(T::U64, expected, expected));
        assert_eq!(
            transfer.range().convert(T::Size).unwrap().range(),
            interval(T::Size, expected, expected)
        );
    }
    assert!(
        matches!(interval(T::Size, 0, max - 1).binary(B::Add, interval(T::Size, 1, 1)).unwrap(),
        CTransfer::NonWrapping(value) if value == interval(T::U64, 1, max))
    );
    assert!(matches!(
        interval(T::Size, 0, max)
            .binary(B::Add, interval(T::Size, 1, 1))
            .unwrap(),
        CTransfer::MayWrap(_)
    ));
    assert!(matches!(
        interval(T::Size, 0, max / 8)
            .binary(B::Multiply, interval(T::Size, 8, 8))
            .unwrap(),
        CTransfer::NonWrapping(_)
    ));
    assert!(matches!(
        interval(T::Size, 0, max / 8 + 1)
            .binary(B::Multiply, interval(T::Size, 8, 8))
            .unwrap(),
        CTransfer::MayWrap(_)
    ));
}

#[test]
fn interval_conversions_preserve_only_proved_bounds_and_mark_modulo_narrowing() {
    assert_eq!(
        interval(T::Int, -3, -1).convert(T::U8).unwrap(),
        CTransfer::MayWrap(interval(T::U8, 253, 255))
    );
    assert_eq!(
        interval(T::Int, -1, 1).convert(T::U8).unwrap(),
        CTransfer::MayWrap(interval(T::U8, 0, 255))
    );
    assert_eq!(
        interval(T::U64, 0, i128::from(i64::MAX))
            .convert(T::I64)
            .unwrap(),
        CTransfer::NonWrapping(interval(T::I64, 0, i128::from(i64::MAX)))
    );
    assert_eq!(
        interval(T::U64, 0, i128::from(i64::MAX) + 1).convert(T::I64),
        Err(E::IntegerRange)
    );
    assert_eq!(
        interval(T::Int, -128, 127).convert(T::I8).unwrap(),
        CTransfer::NonWrapping(interval(T::I8, -128, 127))
    );
    assert_eq!(
        interval(T::Int, -128, 128).convert(T::I8),
        Err(E::IntegerRange)
    );
    assert_eq!(
        interval(T::Int, -3, -1).convert(T::Bool).unwrap().range(),
        interval(T::Bool, 1, 1)
    );
    assert_eq!(
        interval(T::Int, -3, 1).convert(T::Bool).unwrap().range(),
        interval(T::Bool, 0, 1)
    );
}

#[test]
fn non_singleton_undefined_operation_guards_fail_before_using_any_result() {
    let max = i128::from(i32::MAX);
    let min = i128::from(i32::MIN);
    for op in [B::Divide, B::Remainder] {
        assert_eq!(
            interval(T::Int, 1, 2).binary(op, interval(T::Int, -1, 1)),
            Err(E::DivisionByZero)
        );
        assert_eq!(
            interval(T::Int, min, 0).binary(op, interval(T::Int, -2, -1)),
            Err(E::SignedOverflow)
        );
    }
    assert_eq!(
        interval(T::Int, 0, max).binary(B::Add, interval(T::Int, 0, 1)),
        Err(E::SignedOverflow)
    );
    assert_eq!(
        interval(T::Int, min, 0).binary(B::Subtract, interval(T::Int, 0, 1)),
        Err(E::SignedOverflow)
    );
    assert_eq!(
        interval(T::Int, 0, max).binary(B::Multiply, interval(T::Int, 1, 2)),
        Err(E::SignedOverflow)
    );
    assert_eq!(
        interval(T::Int, 0, 1).binary(B::ShiftLeft, interval(T::Int, 0, 31)),
        Err(E::InvalidShift)
    );
    assert_eq!(
        interval(T::Int, -1, 0).binary(B::ShiftLeft, interval(T::Int, 0, 0)),
        Err(E::InvalidShift)
    );
    assert_eq!(
        interval(T::Int, 0, 1).binary(B::ShiftRight, interval(T::U64, 0, i128::from(u64::MAX))),
        Err(E::InvalidShift)
    );
    assert_eq!(
        interval(T::Int, min, 0).unary(U::Negate),
        Err(E::SignedOverflow)
    );
    // Promotions are actual C promotions, not the literal or storage width.
    assert_eq!(
        interval(T::U8, 0, 255)
            .binary(B::Add, interval(T::U8, 0, 255))
            .unwrap()
            .range(),
        interval(T::Int, 0, 510)
    );
    let (_, signed_max) = limits(T::I64).unwrap();
    assert!(
        interval(T::I64, 0, signed_max - 1)
            .binary(B::Add, interval(T::I64, 1, 1))
            .is_ok()
    );
}
