//! Dominating, reversed and stale runtime numeric guards.
use super::numeric_fixture::Fixture;
use super::*;
use crate::dialect::CKnownCall;

#[test]
fn signed_nonzero_division_guard_is_not_replaced_by_an_interval_hull() {
    let mut f = Fixture::new(&[CScalarType::Int]);
    let divide = f.discard(f.binary(CBinaryOperator::Divide, f.int(1), f.input(0)));
    assert_eq!(
        f.check(vec![divide.clone()]),
        Err(CSafetyError::DivisionByZero)
    );
    let condition = f.compare(CBinaryOperator::NotEqual, f.input(0), f.int(0));
    let guarded = f.branch(condition, vec![divide], vec![]);
    f.check(vec![guarded]).unwrap();
}

#[test]
fn reversed_and_post_operation_guards_do_not_authorize_division() {
    for reversed in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Int]);
        let divide = f.discard(f.binary(CBinaryOperator::Divide, f.int(1), f.input(0)));
        let condition = f.compare(CBinaryOperator::Equal, f.input(0), f.int(0));
        let branch = f.branch(
            condition,
            if reversed {
                vec![divide.clone()]
            } else {
                vec![]
            },
            vec![],
        );
        let body = if reversed {
            vec![branch]
        } else {
            vec![divide, branch]
        };
        assert_eq!(f.check(body), Err(CSafetyError::DivisionByZero));
    }
}

#[test]
fn writes_and_opaque_effects_invalidate_guarded_storage_but_not_unexposed_parameters() {
    for exposed in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Int]);
        let parameter = f.values().parameter(f.parameters[0].clone()).unwrap();
        let mut body = Vec::new();
        if exposed {
            body.push(f.discard(f.values().address_of(parameter).unwrap()));
        }
        let call = f
            .values()
            .call_value(f.values().known(CKnownCall::Allocate), vec![f.size(1)])
            .unwrap();
        let branch_body = vec![
            f.discard(call),
            f.discard(f.binary(CBinaryOperator::Divide, f.int(1), f.input(0))),
        ];
        body.push(f.branch(
            f.compare(CBinaryOperator::NotEqual, f.input(0), f.int(0)),
            branch_body,
            vec![],
        ));
        let result = f.check(body);
        if exposed {
            assert_eq!(result, Err(CSafetyError::DivisionByZero));
        } else {
            result.unwrap();
        }
    }
    let mut f = Fixture::new(&[CScalarType::Int]);
    let overwrite = f
        .ast()
        .assign(
            f.values().parameter(f.parameters[0].clone()).unwrap(),
            f.int(0),
        )
        .unwrap();
    let divide = f.discard(f.binary(CBinaryOperator::Divide, f.int(1), f.input(0)));
    let branch = f.branch(
        f.compare(CBinaryOperator::NotEqual, f.input(0), f.int(0)),
        vec![overwrite, divide],
        vec![],
    );
    assert_eq!(f.check(vec![branch]), Err(CSafetyError::DivisionByZero));
}

#[test]
fn both_branch_values_must_join_without_losing_nonzero_exclusions() {
    for zero in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let local = f.local(CScalarType::Int, "divisor");
        let assignment = |value| {
            f.ast()
                .assign(f.values().local(local.clone()).unwrap(), f.int(value))
                .unwrap()
        };
        let yes = assignment(-1);
        let no = assignment(if zero { 0 } else { 1 });
        let branch = f.branch(f.input(0), vec![yes], vec![no]);
        let body = vec![
            f.declare(&local, f.int(1)),
            branch,
            f.discard(f.binary(CBinaryOperator::Divide, f.int(1), f.read(&local))),
        ];
        let result = f.check(body);
        if zero {
            assert_eq!(result, Err(CSafetyError::DivisionByZero));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn short_circuit_and_conditional_guards_apply_before_the_selected_child() {
    let f = Fixture::new(&[CScalarType::Int]);
    let condition = f.compare(CBinaryOperator::NotEqual, f.input(0), f.int(0));
    let divide = f.binary(CBinaryOperator::Divide, f.int(1), f.input(0));
    let logical = f.binary(
        CBinaryOperator::LogicalAnd,
        condition.clone(),
        f.boolean(divide.clone()),
    );
    let conditional = f.values().conditional(condition, divide, f.int(0)).unwrap();
    f.check(vec![f.discard(logical), f.discard(conditional)])
        .unwrap();
}

#[test]
fn signed_overflow_and_shift_boundaries_require_preconditions() {
    let mut f = Fixture::new(&[CScalarType::Int]);
    let add = f.discard(f.binary(CBinaryOperator::Add, f.input(0), f.int(1)));
    assert_eq!(
        f.check(vec![add.clone()]),
        Err(CSafetyError::SignedOverflow)
    );
    let guarded = f.branch(
        f.compare(CBinaryOperator::Less, f.input(0), f.int(i32::MAX)),
        vec![add],
        vec![],
    );
    f.check(vec![guarded]).unwrap();
    let mut f = Fixture::new(&[CScalarType::Int]);
    let bounds = f.binary(
        CBinaryOperator::LogicalAnd,
        f.compare(CBinaryOperator::GreaterEqual, f.input(0), f.int(0)),
        f.compare(CBinaryOperator::Less, f.input(0), f.int(31)),
    );
    let shift = f.discard(f.binary(CBinaryOperator::ShiftLeft, f.int(1), f.input(0)));
    let guarded = f.branch(f.boolean(bounds), vec![shift], vec![]);
    f.check(vec![guarded]).unwrap();
}
