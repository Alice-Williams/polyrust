//! Repeated lexical activations cannot overwrite live owners or pending resets.
use super::{
    contextual_reconstruction::key, counted_fixture::Fixture, storage_fixture::pointer_type, *,
};
use crate::dialect::CKnownCall;

#[test]
fn each_loop_activation_must_drop_and_reset_before_redeclaration() {
    for mode in 0..3 {
        let mut f = Fixture::new();
        let ty = CObjectType::scalar(CScalarType::Int);
        let raw_ty = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
        let raw = f
            .registry
            .register_local(&f.body, key("raw"), raw_ty.clone())
            .unwrap();
        let temporary = f
            .registry
            .register_local(&f.body, key("temporary"), pointer_type(ty.clone()))
            .unwrap();
        let owner = f
            .registry
            .register_local(&f.body, key("owner"), pointer_type(ty.clone()))
            .unwrap();
        f.registry.register_local_owner(&owner).unwrap();
        let descriptor = f
            .registry
            .register_allocation(&f.body, key("object"), ty, CAllocatorSource::Default)
            .unwrap();
        let success = f.child(&f.body.clone(), "success");
        let failure = f.child(&f.body.clone(), "failure");
        let e = f.expressions();
        let s = f.statements();
        let null = e
            .literal(CLiteral::NullPointer(
                CNullPointer::new(owner.ty().clone()).unwrap(),
            ))
            .unwrap();
        let condition = e
            .numeric_conversion(
                CScalarType::Bool,
                e.pointer_test(CPointerTest::IsNonNull(Box::new(f.read(&raw))))
                    .unwrap(),
            )
            .unwrap();
        let mut iteration = vec![
            f.declare(
                &raw,
                e.call_value(e.known(CKnownCall::Allocate), vec![f.size(4)])
                    .unwrap(),
            ),
            s.if_statement(
                condition,
                s.block(success, vec![]).unwrap(),
                s.block(failure, vec![s.return_statement(None).unwrap()])
                    .unwrap(),
            )
            .unwrap(),
            f.declare(
                &temporary,
                e.allocation_restore(descriptor, f.read(&raw)).unwrap(),
            ),
            f.declare(&owner, null.clone()),
            s.assign(
                e.dereference(f.read(&temporary)).unwrap(),
                e.literal(CLiteral::Signed(CSignedLiteral::Int(7))).unwrap(),
            )
            .unwrap(),
            s.assign(e.local(owner.clone()).unwrap(), f.read(&temporary))
                .unwrap(),
        ];
        if mode < 2 {
            iteration.push(
                s.evaluate(
                    e.call_effect(
                        e.known(CKnownCall::Release),
                        vec![e.object_to_void(raw_ty, f.read(&owner)).unwrap()],
                    )
                    .unwrap(),
                )
                .unwrap(),
            );
        }
        if mode == 0 {
            iteration.push(s.assign(e.local(owner).unwrap(), null).unwrap());
        }
        iteration.push(f.step());
        let source = f.source(vec![
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(3)),
            f.iteration(iteration),
        ]);
        f.registry
            .check_package_structure(std::slice::from_ref(&source))
            .unwrap();
        let result = f.registry.check_storage_paths(&[source]);
        if mode == 0 {
            assert_eq!(result, Ok(()));
        } else {
            assert!(
                matches!(
                    result,
                    Err(CSafetyError::UnprovedOwnership | CSafetyError::UninitializedStorage)
                ),
                "mode={mode}: {result:?}"
            );
        }
    }
}
