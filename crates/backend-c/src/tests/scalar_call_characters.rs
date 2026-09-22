//! New U32 signatures retain every original call/storage prerequisite.
use super::{ScalarCalls, fixture::Fixture, shape};
use crate::ast::*;
use std::collections::BTreeSet;

#[test]
fn character_conditionals_keep_edges_in_every_child_and_reject_missing_bodies() {
    for child in 0..3 {
        for defined in [false, true] {
            let f = Fixture::with_result(
                &[vec![], vec![], vec![]],
                CObjectType::scalar(CScalarType::U32),
            );
            let e = f.values();
            let scalar = e
                .literal(CLiteral::Unsigned(CUnsignedLiteral::U32(0x10ffff)))
                .unwrap();
            let call = f.call(2, vec![]);
            let condition = if child == 0 {
                e.numeric_conversion(
                    CScalarType::Bool,
                    e.binary(CBinaryOperator::Equal, call.clone(), scalar.clone())
                        .unwrap(),
                )
                .unwrap()
            } else {
                e.literal(CLiteral::Bool(true)).unwrap()
            };
            let value = e
                .conditional(
                    condition,
                    if child == 1 {
                        call.clone()
                    } else {
                        scalar.clone()
                    },
                    if child == 2 { call } else { scalar.clone() },
                )
                .unwrap();
            let body = f
                .statements(1)
                .block(f.scopes[1].clone(), f.returning(1, value.clone()))
                .unwrap();
            assert_eq!(
                shape::dependencies(&f.functions[1], &body),
                Some(BTreeSet::from([f.functions[2].clone()]))
            );
            let mut bodies = vec![f.returning(0, f.call(1, vec![])), f.returning(1, value)];
            if defined {
                bodies.push(f.returning(2, scalar));
            }
            let summary = ScalarCalls::derive(&[f.source(bodies)]);
            for function in &f.functions {
                assert_eq!(
                    summary.accepts(&e.direct(function.clone()).unwrap()),
                    defined
                );
            }
        }
    }
}

#[test]
fn character_results_do_not_hide_indirect_or_foreign_effects_or_pointer_parameters() {
    for indirect in [false, true] {
        let f = Fixture::with_result(&[vec![], vec![]], CObjectType::scalar(CScalarType::U32));
        let e = f.values();
        let scalar = e
            .literal(CLiteral::Unsigned(CUnsignedLiteral::U32(0x10000)))
            .unwrap();
        let effect = if indirect {
            e.call_value(
                e.indirect(
                    e.function_address(f.functions[1].clone()).unwrap(),
                    f.functions[1].clone(),
                )
                .unwrap(),
                vec![],
            )
            .unwrap()
        } else {
            e.call_value(
                e.known(crate::dialect::CKnownCall::Allocate),
                vec![
                    e.literal(CLiteral::Unsigned(CUnsignedLiteral::Size(8)))
                        .unwrap(),
                ],
            )
            .unwrap()
        };
        let mut caller = vec![f.statements(0).discard(effect).unwrap()];
        caller.extend(f.returning(0, scalar.clone()));
        let source = f.source(vec![caller, f.returning(1, scalar)]);
        let summary = ScalarCalls::derive(std::slice::from_ref(&source));
        assert!(!summary.accepts(&e.direct(f.functions[0].clone()).unwrap()));
        assert!(summary.accepts(&e.direct(f.functions[1].clone()).unwrap()));
        assert!(f.registry.check_storage_paths(&[source]).is_err());
    }
    let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::scalar(
        CScalarType::U32,
    ))));
    let f = Fixture::with_result(&[vec![pointer]], CObjectType::scalar(CScalarType::U32));
    let value = f
        .values()
        .literal(CLiteral::Unsigned(CUnsignedLiteral::U32(0)))
        .unwrap();
    let summary = ScalarCalls::derive(&[f.source(vec![f.returning(0, value)])]);
    assert!(!summary.accepts(&f.values().direct(f.functions[0].clone()).unwrap()));
}
