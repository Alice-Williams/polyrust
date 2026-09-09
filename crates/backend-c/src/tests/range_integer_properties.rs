//! Finite exhaustive concrete evaluation checks interval soundness independently.
use super::{CScalarRange as Range, integer::IntegerRange};
use crate::ast::CUnaryOperator as U;
use crate::ast::{CBinaryOperator as B, CScalarType as T};
use crate::ownership::constants::{CInteger, CNumber};

pub(super) const OPS: [B; 18] = [
    B::Add,
    B::Subtract,
    B::Multiply,
    B::Divide,
    B::Remainder,
    B::ShiftLeft,
    B::ShiftRight,
    B::BitAnd,
    B::BitOr,
    B::BitXor,
    B::Equal,
    B::NotEqual,
    B::Less,
    B::LessEqual,
    B::Greater,
    B::GreaterEqual,
    B::LogicalAnd,
    B::LogicalOr,
];
pub(super) const TYPES: [T; 13] = [
    T::Bool,
    T::PlainChar,
    T::Int,
    T::I8,
    T::U8,
    T::I16,
    T::U16,
    T::I32,
    T::U32,
    T::I64,
    T::U64,
    T::Size,
    T::F64,
];
pub(super) fn number(ty: T, value: i128) -> CNumber {
    CNumber::Integer(CInteger::checked(ty, value).unwrap())
}
pub(super) fn interval(ty: T, min: i128, max: i128) -> Range {
    Range::Integer(IntegerRange::checked(ty, min, max).unwrap())
}

#[test]
fn bounded_unary_and_conversion_results_contain_every_concrete_input() {
    for ty in [T::Int, T::U32, T::Bool] {
        let (floor, ceiling) = if ty == T::Int {
            (-4, 4)
        } else if ty == T::Bool {
            (0, 1)
        } else {
            (0, 8)
        };
        for min in floor..=ceiling {
            for max in min..=ceiling {
                let range = interval(ty, min, max);
                for op in [U::Negate, U::BitNot, U::LogicalNot] {
                    if let Ok(result) = range.unary(op) {
                        for value in min..=max {
                            assert!(
                                result
                                    .range()
                                    .contains(number(ty, value).unary(op).unwrap())
                            );
                        }
                    }
                }
                for destination in TYPES {
                    if let Ok(result) = range.convert(destination) {
                        for value in min..=max {
                            assert!(
                                result
                                    .range()
                                    .contains(number(ty, value).convert(destination).unwrap())
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn exhaustive_small_integer_intervals_contain_every_result_and_never_hide_undefined_pairs() {
    for ty in [T::Int, T::U32, T::Bool] {
        let (floor, ceiling) = match ty {
            T::Int => (-3, 3),
            T::U32 => (0, 6),
            T::Bool => (0, 1),
            _ => unreachable!(),
        };
        let mut accepted = [0_usize; 18];
        for min in floor..=ceiling {
            for max in min..=ceiling {
                for other_min in floor..=ceiling {
                    for other_max in other_min..=ceiling {
                        for (index, op) in OPS.iter().copied().enumerate() {
                            let Ok(transfer) = interval(ty, min, max)
                                .binary(op, interval(ty, other_min, other_max))
                            else {
                                continue;
                            };
                            accepted[index] += 1;
                            for a in min..=max {
                                for b in other_min..=other_max {
                                    let expected = number(ty, a).binary(op, number(ty, b))
                                        .unwrap_or_else(|error| panic!("accepted unsafe interval {ty:?} {op:?}: {a}, {b}: {error:?}"));
                                    assert!(
                                        transfer.range().contains(expected),
                                        "{ty:?} [{min},{max}] {op:?} [{other_min},{other_max}] misses {a},{b}: {transfer:?}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        for (op, count) in OPS.into_iter().zip(accepted) {
            if op.result_type(ty, ty).is_ok() {
                assert!(count > 0, "no positive {ty:?} {op:?}");
            }
        }
    }
}

#[test]
fn useful_bounded_integer_results_do_not_collapse_to_the_full_type_domain() {
    let cases = [
        (B::Add, (1, 3), (4, 8), (5, 11)),
        (B::Subtract, (1, 3), (4, 8), (-7, -1)),
        (B::Multiply, (-3, 4), (-2, 5), (-15, 20)),
        (B::Divide, (-9, 9), (2, 3), (-4, 4)),
        (B::Remainder, (-9, 9), (2, 3), (-2, 2)),
        (B::BitAnd, (0, 255), (15, 15), (0, 15)),
        (B::ShiftRight, (-16, 16), (1, 3), (-8, 8)),
        (B::ShiftLeft, (1, 3), (1, 3), (2, 24)),
    ];
    for (op, (a, b), (c, d), (min, max)) in cases {
        assert_eq!(
            interval(T::Int, a, b)
                .binary(op, interval(T::Int, c, d))
                .unwrap()
                .range(),
            interval(T::Int, min, max)
        );
    }
    assert_eq!(
        interval(T::Int, 0, 2)
            .binary(B::Less, interval(T::Int, 3, 5))
            .unwrap()
            .range(),
        interval(T::Int, 1, 1)
    );
}
