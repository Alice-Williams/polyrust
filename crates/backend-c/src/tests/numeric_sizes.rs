//! A Size spelling or a later guard cannot launder possibly wrapped arithmetic.
use super::numeric_fixture::Fixture;
use super::*;
use crate::dialect::CKnownCall;

fn allocate(f: &Fixture, bytes: CValue) -> CStatement {
    f.discard(
        f.values()
            .call_value(f.values().known(CKnownCall::Allocate), vec![bytes])
            .unwrap(),
    )
}
fn product(f: &Fixture) -> CValue {
    f.binary(CBinaryOperator::Multiply, f.input(0), f.size(8))
}
fn guard(f: &Fixture) -> CValue {
    f.compare(
        CBinaryOperator::LessEqual,
        f.input(0),
        f.binary(CBinaryOperator::Divide, f.size(u64::MAX), f.size(8)),
    )
}

#[test]
fn checked_product_guard_must_precede_the_operation() {
    for guarded in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let call = allocate(&f, product(&f));
        let body = if guarded {
            vec![f.branch(guard(&f), vec![call], vec![])]
        } else {
            vec![call]
        };
        let result = f.check(body);
        if guarded {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CSafetyError::UnprovedSizeArithmetic));
        }
    }
}

#[test]
fn materializing_before_a_guard_retains_the_possible_wrap_history() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let bytes = f.local(CScalarType::Size, "bytes");
    let call = allocate(&f, f.read(&bytes));
    let branch = f.branch(guard(&f), vec![call], vec![]);
    assert_eq!(
        f.check(vec![f.declare(&bytes, product(&f)), branch]),
        Err(CSafetyError::UnprovedSizeArithmetic)
    );
}

#[test]
fn materializing_after_a_guard_carries_the_checked_result() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let bytes = f.local(CScalarType::Size, "bytes");
    let assignment = f
        .ast()
        .assign(f.values().local(bytes.clone()).unwrap(), product(&f))
        .unwrap();
    let call = allocate(&f, f.read(&bytes));
    let branch = f.branch(guard(&f), vec![assignment, call], vec![]);
    f.check(vec![f.declare(&bytes, f.size(0)), branch]).unwrap();
}

#[test]
fn an_opaque_write_cannot_erase_possible_wrapped_history() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let bytes = f.local(CScalarType::Size, "bytes");
    let address = f.discard(
        f.values()
            .address_of(f.values().local(bytes.clone()).unwrap())
            .unwrap(),
    );
    let checked = f.check(vec![
        f.declare(&bytes, product(&f)),
        address,
        allocate(&f, f.size(1)),
        allocate(&f, f.read(&bytes)),
    ]);
    assert_eq!(checked, Err(CSafetyError::UnprovedSizeArithmetic));
}

#[test]
fn modulo_conversions_and_mixed_signedness_guards_do_not_have_false_preimages() {
    let mut f = Fixture::new(&[CScalarType::U64]);
    let narrow = f
        .values()
        .numeric_conversion(CScalarType::U8, f.input(0))
        .unwrap();
    let condition = f.compare(CBinaryOperator::NotEqual, narrow, f.int(0));
    let cast = f.discard(
        f.values()
            .numeric_conversion(CScalarType::Int, f.input(0))
            .unwrap(),
    );
    let branch = f.branch(condition, vec![cast], vec![]);
    assert_eq!(f.check(vec![branch]), Err(CSafetyError::IntegerRange));

    let mut f = Fixture::new(&[CScalarType::Int]);
    let bound = f
        .values()
        .literal(CLiteral::Unsigned(CUnsignedLiteral::U32(u32::MAX)))
        .unwrap();
    let condition = f.compare(CBinaryOperator::Less, f.input(0), bound);
    let negate = f.discard(
        f.values()
            .unary(CUnaryOperator::Negate, f.input(0))
            .unwrap(),
    );
    let branch = f.branch(condition, vec![negate], vec![]);
    assert_eq!(f.check(vec![branch]), Err(CSafetyError::SignedOverflow));
}

#[test]
fn division_guards_must_exclude_both_zero_and_signed_minimum_over_minus_one() {
    for exclude_min in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Int, CScalarType::Int]);
        let mut condition = f.compare(CBinaryOperator::NotEqual, f.input(1), f.int(0));
        if exclude_min {
            condition = f.boolean(f.binary(
                CBinaryOperator::LogicalAnd,
                condition,
                f.compare(CBinaryOperator::NotEqual, f.input(0), f.int(i32::MIN)),
            ));
        }
        let divide = f.discard(f.binary(CBinaryOperator::Divide, f.input(0), f.input(1)));
        let branch = f.branch(condition, vec![divide], vec![]);
        let result = f.check(vec![branch]);
        if exclude_min {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CSafetyError::SignedOverflow));
        }
    }
}
