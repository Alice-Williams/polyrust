//! Actual guards authorize current size terms, never stale or merely similar values.
use super::{numeric_fixture::Fixture, *};
use crate::dialect::CKnownCall;
use CBinaryOperator as B;

fn allocate(f: &Fixture, value: CValue) -> CStatement {
    f.discard(
        f.values()
            .call_value(f.values().known(CKnownCall::Allocate), vec![value])
            .unwrap(),
    )
}
fn bound(f: &Fixture, operation: B) -> CValue {
    f.binary(
        if operation == B::Add {
            B::Subtract
        } else {
            B::Divide
        },
        f.size(u64::MAX),
        f.input(1),
    )
}
fn guard(f: &Fixture, operation: B) -> CValue {
    let comparison = f.compare(B::LessEqual, f.input(0), bound(f, operation));
    if operation == B::Add {
        comparison
    } else {
        f.boolean(f.binary(
            B::LogicalAnd,
            f.compare(B::NotEqual, f.input(1), f.size(0)),
            comparison,
        ))
    }
}
fn calculation(f: &Fixture, operation: B) -> CValue {
    f.binary(operation, f.input(0), f.input(1))
}

#[test]
fn two_unknown_sizes_use_a_dominating_algebraic_bound() {
    for operation in [B::Add, B::Multiply] {
        for guarded in [false, true] {
            let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
            let call = allocate(&f, calculation(&f, operation));
            let body = if guarded {
                vec![f.branch(guard(&f, operation), vec![call], vec![])]
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
}

#[test]
fn strict_reversed_and_false_edges_preserve_the_mathematical_implication() {
    for (operator, swapped, branch_truth) in [
        (B::Less, false, true),
        (B::GreaterEqual, true, true),
        (B::Greater, false, false),
    ] {
        let mut f = Fixture::new(&[CScalarType::U64, CScalarType::Size]);
        let (left, right) = if swapped {
            (bound(&f, B::Add), f.input(0))
        } else {
            (f.input(0), bound(&f, B::Add))
        };
        let condition = f.compare(operator, left, right);
        let call = allocate(&f, calculation(&f, B::Add));
        let (yes, no) = if branch_truth {
            (vec![call], vec![])
        } else {
            (vec![], vec![call])
        };
        let branch = f.branch(condition, yes, no);
        f.check(vec![branch]).unwrap();
    }
}

#[test]
fn the_opposite_edge_or_an_unguarded_join_cannot_reuse_the_relation() {
    for opposite in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
        let call = allocate(&f, calculation(&f, B::Add));
        let branch = f.branch(
            guard(&f, B::Add),
            vec![],
            if opposite { vec![call.clone()] } else { vec![] },
        );
        let body = if opposite {
            vec![branch]
        } else {
            vec![branch, call]
        };
        assert_eq!(f.check(body), Err(CSafetyError::UnprovedSizeArithmetic));
    }
}

#[test]
fn writes_and_opaque_exposure_kill_relations_but_unexposed_values_survive() {
    for changed in [0, 1, 2, 3] {
        let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
        let parameter = f.values().parameter(f.parameters[1].clone()).unwrap();
        let mut prefix = vec![];
        if changed == 2 {
            prefix.push(f.discard(f.values().address_of(parameter.clone()).unwrap()));
        }
        let action = if changed < 2 {
            f.ast()
                .assign(
                    f.values().parameter(f.parameters[changed].clone()).unwrap(),
                    f.size(u64::MAX),
                )
                .unwrap()
        } else {
            allocate(&f, f.size(1))
        };
        let call = allocate(&f, calculation(&f, B::Add));
        prefix.push(f.branch(guard(&f, B::Add), vec![action, call], vec![]));
        let result = f.check(prefix);
        if changed == 3 {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CSafetyError::UnprovedSizeArithmetic));
        }
    }
}

#[test]
fn a_later_relation_does_not_erase_earlier_wrap_history() {
    let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
    let bytes = f.local(CScalarType::Size, "bytes");
    let call = allocate(&f, f.read(&bytes));
    let branch = f.branch(guard(&f, B::Add), vec![call], vec![]);
    assert_eq!(
        f.check(vec![f.declare(&bytes, calculation(&f, B::Add)), branch]),
        Err(CSafetyError::UnprovedSizeArithmetic)
    );
}

#[test]
fn division_bound_itself_requires_a_preceding_nonzero_guard() {
    let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
    let condition = f.compare(B::LessEqual, f.input(0), bound(&f, B::Multiply));
    let call = allocate(&f, calculation(&f, B::Multiply));
    let branch = f.branch(condition, vec![call], vec![]);
    assert_eq!(f.check(vec![branch]), Err(CSafetyError::DivisionByZero));
}

#[test]
fn implicit_write_bytes_product_uses_the_actual_call_point_guard() {
    for guarded in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
        let row = super::known_call_fixtures::rows()
            .into_iter()
            .find(|row| row.call == CKnownCall::WriteBytes)
            .unwrap();
        let mut arguments = row.arguments(&f.values());
        arguments[1] = f.input(0);
        arguments[2] = f.input(1);
        // Null pointer/stream obligations intentionally remain for storage/call safety.
        let call = f.discard(
            f.values()
                .call_value(f.values().known(CKnownCall::WriteBytes), arguments)
                .unwrap(),
        );
        let body = if guarded {
            vec![f.branch(guard(&f, B::Multiply), vec![call], vec![])]
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
fn boundary_oracle_checks_both_guard_identities_without_modulo_arithmetic() {
    let values = [
        0,
        1,
        2,
        3,
        255,
        u32::MAX as u64,
        (u32::MAX as u64) + 1,
        u64::MAX / 2,
        u64::MAX - 1,
        u64::MAX,
    ];
    for a in values {
        for b in values {
            assert_eq!(
                a <= u64::MAX - b,
                u128::from(a) + u128::from(b) <= u128::from(u64::MAX)
            );
            if let Some(bound) = u64::MAX.checked_div(b) {
                assert_eq!(
                    a <= bound,
                    u128::from(a) * u128::from(b) <= u128::from(u64::MAX)
                );
            }
        }
    }
}
