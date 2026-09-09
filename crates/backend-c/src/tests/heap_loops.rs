//! Rebinding a producing site never revives an earlier typed/interior pointer.
use super::{
    contextual_reconstruction::key, counted_fixture::Fixture, storage_fixture::pointer_type, *,
};
use crate::dialect::CKnownCall;

#[test]
fn repeated_typed_allocations_reset_storage_and_expire_retained_aliases() {
    for interior in [false, true] {
        for mode in 0..3 {
            let mut f = Fixture::new();
            let scalar = CObjectType::scalar(CScalarType::Int);
            let object = if interior {
                CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap()
            } else {
                scalar.clone()
            };
            let raw_ty = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
            let raw = f
                .registry
                .register_local(&f.body, key("raw"), raw_ty)
                .unwrap();
            let typed = f
                .registry
                .register_local(&f.body, key("typed"), pointer_type(object.clone()))
                .unwrap();
            let old = f
                .registry
                .register_local(&f.scope, key("old"), pointer_type(scalar))
                .unwrap();
            let descriptor = f
                .registry
                .register_allocation(&f.body, key("object"), object, CAllocatorSource::Default)
                .unwrap();
            let success = f.child(&f.body.clone(), "success");
            let failure = f.child(&f.body.clone(), "failure");
            let later = f.child(&f.body.clone(), "later");
            let first = f.child(&f.body.clone(), "first");
            let write_first = f.child(&f.body.clone(), "write_first");
            let write_later = f.child(&f.body.clone(), "write_later");
            let e = f.expressions();
            let s = f.statements();
            let nonnull = e
                .numeric_conversion(
                    CScalarType::Bool,
                    e.pointer_test(CPointerTest::IsNonNull(Box::new(f.read(&raw))))
                        .unwrap(),
                )
                .unwrap();
            let next = e
                .numeric_conversion(
                    CScalarType::Bool,
                    e.binary(CBinaryOperator::Greater, f.read(&f.counter), f.size(0))
                        .unwrap(),
                )
                .unwrap();
            let selected = e.dereference(f.read(&typed)).unwrap();
            let selected = if interior {
                e.index(CIndexBase::Array(Box::new(selected)), f.size(1))
                    .unwrap()
            } else {
                selected
            };
            let read_old = s
                .discard(e.read(e.dereference(f.read(&old)).unwrap()).unwrap())
                .unwrap();
            let write = s
                .assign(
                    selected.clone(),
                    e.literal(CLiteral::Signed(CSignedLiteral::Int(7))).unwrap(),
                )
                .unwrap();
            let initial = e
                .numeric_conversion(
                    CScalarType::Bool,
                    e.binary(CBinaryOperator::Equal, f.read(&f.counter), f.size(0))
                        .unwrap(),
                )
                .unwrap();
            let iteration = vec![
                f.declare(
                    &raw,
                    e.call_value(e.known(CKnownCall::Allocate), vec![f.size(8)])
                        .unwrap(),
                ),
                s.if_statement(
                    nonnull,
                    s.block(success, vec![]).unwrap(),
                    s.block(failure, vec![s.return_statement(None).unwrap()])
                        .unwrap(),
                )
                .unwrap(),
                f.declare(
                    &typed,
                    e.allocation_restore(descriptor, f.read(&raw)).unwrap(),
                ),
                s.if_statement(
                    next,
                    s.block(later, if mode == 1 { vec![read_old] } else { vec![] })
                        .unwrap(),
                    s.block(first, vec![]).unwrap(),
                )
                .unwrap(),
                s.if_statement(
                    initial,
                    s.block(write_first, vec![write.clone()]).unwrap(),
                    s.block(write_later, if mode == 2 { vec![] } else { vec![write] })
                        .unwrap(),
                )
                .unwrap(),
                s.discard(e.read(selected.clone()).unwrap()).unwrap(),
                s.assign(
                    e.local(old.clone()).unwrap(),
                    e.address_of(selected).unwrap(),
                )
                .unwrap(),
                s.evaluate(
                    e.call_effect(e.known(CKnownCall::Release), vec![f.read(&raw)])
                        .unwrap(),
                )
                .unwrap(),
                f.step(),
            ];
            let null = e
                .literal(CLiteral::NullPointer(
                    CNullPointer::new(old.ty().clone()).unwrap(),
                ))
                .unwrap();
            let source = f.source(vec![
                f.declare(&old, null),
                f.declare(&f.counter, f.size(0)),
                f.declare(&f.bound, f.size(2)),
                f.iteration(iteration),
            ]);
            f.registry
                .check_numeric_flow(std::slice::from_ref(&source))
                .unwrap();
            let result = f.registry.check_storage_paths(&[source]);
            if mode != 0 {
                assert!(
                    result.is_err(),
                    "interior={interior}, mode={mode} revived an old pointer or initialized storage"
                );
            } else {
                assert_eq!(result, Ok(()), "interior={interior}");
            }
        }
    }
}
