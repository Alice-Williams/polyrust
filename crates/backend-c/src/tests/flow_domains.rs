//! Independent finite-set oracle for flow unions, intersections and transfers.
use super::*;
use crate::ast::{CBinaryOperator as B, CScalarType as T};
use crate::ownership::constants::{CInteger, CNumber};

fn integer(value: i128) -> CNumber {
    CNumber::Integer(CInteger::checked(T::Int, value).unwrap())
}
fn subset(mask: u32) -> NumericDomain {
    let mut domain = NumericDomain::empty(T::Int);
    for index in 0..5 {
        if mask & (1 << index) != 0 {
            domain = domain
                .join(&NumericDomain::exact(integer(i128::from(index) - 2)))
                .unwrap();
        }
    }
    domain
}

#[test]
fn every_small_finite_set_join_intersection_and_exclusion_matches_membership() {
    for left in 0..32 {
        for right in 0..32 {
            let a = subset(left);
            let b = subset(right);
            let union = a.join(&b).unwrap();
            let intersection = a.intersect(&b).unwrap();
            assert_eq!(union, subset(left | right));
            assert_eq!(intersection, subset(left & right));
            for value in -3..=3 {
                let exact = integer(value);
                assert_eq!(
                    union.contains(exact),
                    a.contains(exact) || b.contains(exact)
                );
                assert_eq!(
                    intersection.contains(exact),
                    a.contains(exact) && b.contains(exact)
                );
                assert!(!a.exclude_integer(value).unwrap().contains(exact));
            }
        }
    }
}

#[test]
fn interval_union_transfer_contains_every_exact_defined_operand_pair() {
    let operators = [
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
    ];
    for operator in operators {
        let mut accepted_nonempty = false;
        for left in 1..32 {
            for right in 1..32 {
                let a = subset(left);
                let b = subset(right);
                let Ok(result) = a.binary(operator, &b) else {
                    continue;
                };
                accepted_nonempty = true;
                for left in -2..=2 {
                    for right in -2..=2 {
                        if a.contains(integer(left)) && b.contains(integer(right)) {
                            let exact = integer(left)
                                .binary(operator, integer(right))
                                .expect("an admitted pair must have defined arithmetic");
                            assert!(
                                result.domain.contains(exact),
                                "{operator:?}: {left}, {right}"
                            );
                        }
                    }
                }
            }
        }
        assert!(accepted_nonempty, "no positive witness for {operator:?}");
    }
}

#[test]
fn signed_nonzero_guard_keeps_the_hole_and_widening_only_loses_precision() {
    let broad = NumericDomain::full(T::Int).unwrap();
    let nonzero = broad.exclude_integer(0).unwrap();
    assert_eq!(nonzero.truth(), Some(true));
    assert!(!nonzero.contains(integer(0)));
    let numerator = NumericDomain::exact(integer(1));
    numerator.binary(B::Divide, &nonzero).unwrap();
    assert!(numerator.binary(B::Divide, &broad).is_err());
    let a = subset(0b01010);
    let b = subset(0b10001);
    assert_eq!(a.widen(&a).unwrap(), a);
    assert_eq!(a.widen(&b).unwrap(), broad);
    assert_eq!(NumericDomain::empty(T::Int).widen(&a).unwrap(), a);
}

#[test]
fn floating_intersection_retains_signed_zero_and_nan_classes() {
    let positive = NumericDomain::exact(CNumber::Double(0.0));
    let negative = NumericDomain::exact(CNumber::Double(-0.0));
    assert!(positive.intersect(&negative).unwrap().is_empty());
    let zeros = positive.join(&negative).unwrap();
    assert_eq!(zeros.exact_value(), None);
    assert!(zeros.contains(CNumber::Double(0.0)));
    assert!(zeros.contains(CNumber::Double(-0.0)));
    assert_eq!(zeros.intersect(&negative).unwrap(), negative);
    let a = NumericDomain::exact(CNumber::Double(f64::from_bits(0x7ff8000000000001)));
    let b = NumericDomain::exact(CNumber::Double(f64::from_bits(0xfff8000000000002)));
    let joined = a.join(&b).unwrap();
    assert!(joined.contains(CNumber::Double(f64::from_bits(0x7ff8000000000001))));
    assert!(joined.contains(CNumber::Double(f64::from_bits(0xfff8000000000002))));
    assert!(!a.intersect(&b).unwrap().is_empty());
    assert!(
        joined
            .restrict_float(f64::NEG_INFINITY, f64::INFINITY, false)
            .unwrap()
            .is_empty()
    );
    let unknown = NumericDomain::full(T::F64).unwrap();
    let finite = unknown.restrict_float(-1.0, 1.0, false).unwrap();
    assert!(!finite.may_nan());
    assert!(!finite.contains(CNumber::Double(f64::INFINITY)));
    assert!(finite.contains(CNumber::Double(-0.0)));
    assert!(!unknown.only_nan().unwrap().contains(CNumber::Double(0.0)));
}

#[test]
fn mismatched_types_and_empty_domains_do_not_invent_values() {
    let a = NumericDomain::full(T::Int).unwrap();
    let b = NumericDomain::full(T::I32).unwrap();
    assert!(a.join(&b).is_err());
    assert!(a.intersect(&b).is_err());
    assert!(a.restrict_float(0.0, 1.0, false).is_err());
    let empty = NumericDomain::empty(T::Int);
    assert!(empty.binary(B::Add, &a).unwrap().domain.is_empty());
    assert_eq!(empty.truth(), None);
    assert!(empty.convert(T::F64).unwrap().domain.is_empty());
}
