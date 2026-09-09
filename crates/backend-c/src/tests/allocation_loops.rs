//! Repeated actual call sites are activations, not resurrected pointer identities.
use super::{contextual_reconstruction::key, counted_fixture::Fixture, *};
use crate::dialect::CKnownCall;

#[test]
fn repeated_allocation_requires_release_and_never_revives_a_retained_copy() {
    for mode in 0..3 {
        let mut f = Fixture::new();
        let raw = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
        let pointer = f
            .registry
            .register_local(&f.body, key("pointer"), raw.clone())
            .unwrap();
        let old = f
            .registry
            .register_local(&f.scope, key("old"), raw.clone())
            .unwrap();
        let yes = f.child(&f.body.clone(), "yes");
        let no = f.child(&f.body.clone(), "no");
        let e = f.expressions();
        let s = f.statements();
        let call = e
            .call_value(e.known(CKnownCall::Allocate), vec![f.size(8)])
            .unwrap();
        let release = s
            .evaluate(
                e.call_effect(e.known(CKnownCall::Release), vec![f.read(&pointer)])
                    .unwrap(),
            )
            .unwrap();
        let read = s.discard(f.read(&old)).unwrap();
        let guard = e
            .numeric_conversion(
                CScalarType::Bool,
                e.binary(CBinaryOperator::Greater, f.read(&f.counter), f.size(0))
                    .unwrap(),
            )
            .unwrap();
        let mut body = vec![
            f.declare(&pointer, call),
            s.if_statement(
                guard,
                s.block(yes, if mode == 2 { vec![read] } else { vec![] })
                    .unwrap(),
                s.block(no, vec![]).unwrap(),
            )
            .unwrap(),
            s.assign(e.local(old.clone()).unwrap(), f.read(&pointer))
                .unwrap(),
        ];
        if mode != 1 {
            body.push(release);
        }
        body.push(f.step());
        let null = e
            .literal(CLiteral::NullPointer(CNullPointer::new(raw).unwrap()))
            .unwrap();
        let source = f.source(vec![
            f.declare(&old, null),
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(2)),
            f.iteration(body),
        ]);
        f.registry
            .check_numeric_flow(std::slice::from_ref(&source))
            .unwrap();
        let result = f.registry.check_storage_paths(&[source]);
        if mode == 0 {
            assert_eq!(result, Ok(()));
        } else {
            assert!(result.is_err(), "mode={mode} unexpectedly accepted");
        }
    }
}
