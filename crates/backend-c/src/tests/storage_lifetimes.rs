//! Crossed lexical exits invalidate every retained automatic address.
use super::{
    contextual_reconstruction::{fixture_return, key, package},
    numeric_fixture::Fixture,
    storage_fixture::*,
    *,
};

#[test]
fn normal_and_cleanup_exits_expire_copied_stack_pointers() {
    for cleanup in [false, true] {
        for outside in [false, true] {
            let mut f = Fixture::new(&[]);
            let child = f
                .registry
                .register_scope(&f.function, Some(&f.scope), key("child"))
                .unwrap();
            let value = f
                .registry
                .register_local(&child, key("value"), CObjectType::scalar(CScalarType::Int))
                .unwrap();
            let pointer = local(&mut f, pointer_type(value.ty().clone()), "pointer");
            let label = cleanup.then(|| {
                f.registry
                    .register_cleanup_exit(&f.scope, key("done"))
                    .unwrap()
            });
            let null = f
                .values()
                .literal(CLiteral::NullPointer(
                    CNullPointer::new(pointer.ty().clone()).unwrap(),
                ))
                .unwrap();
            let mut inner = vec![
                f.declare(&value, f.int(7)),
                f.ast()
                    .assign(
                        f.values().local(pointer.clone()).unwrap(),
                        address(&f, f.values().local(value).unwrap()),
                    )
                    .unwrap(),
                f.discard(pointed_read(&f, f.read(&pointer))),
            ];
            if let Some(label) = &label {
                inner.push(f.ast().cleanup_jump(label.clone()).unwrap());
            }
            let mut body = vec![
                f.declare(&pointer, null),
                f.ast()
                    .nested_block(f.ast().block(child, inner).unwrap())
                    .unwrap(),
            ];
            let last = if outside {
                f.discard(pointed_read(&f, f.read(&pointer)))
            } else {
                f.discard(f.int(0))
            };
            body.push(if let Some(label) = label {
                f.ast().label(label, last).unwrap()
            } else {
                last
            });
            check(
                &f,
                body,
                if outside {
                    Err(CSafetyError::ExpiredStorage)
                } else {
                    Ok(())
                },
            );
        }
    }
}

#[test]
fn returning_an_automatic_address_is_not_a_live_external_pointer() {
    let ty = CObjectType::scalar(CScalarType::Int);
    let pointer = pointer_type(ty.clone());
    let (mut registry, file, function, scope) =
        fixture_return(CReturnType::Value(CReturnValue::new(pointer).unwrap()));
    let value = registry.register_local(&scope, key("value"), ty).unwrap();
    let expressions = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let source = package(
        &registry,
        file,
        function,
        scope,
        vec![
            statements
                .declare(
                    value.clone(),
                    Some(expressions.zero_initializer(value.ty().clone()).unwrap()),
                )
                .unwrap(),
            statements
                .return_statement(Some(
                    expressions
                        .address_of(expressions.local(value).unwrap())
                        .unwrap(),
                ))
                .unwrap(),
        ],
    );
    assert_eq!(
        registry.check_storage_paths(&[source]),
        Err(CSafetyError::AutomaticAddressEscape)
    );
}

#[test]
fn a_pointer_to_an_ancestor_survives_a_nested_scope_exit() {
    let mut f = Fixture::new(&[]);
    let child = f
        .registry
        .register_scope(&f.function, Some(&f.scope), key("child"))
        .unwrap();
    let value = f.local(CScalarType::Int, "value");
    let pointer = local(&mut f, pointer_type(value.ty().clone()), "pointer");
    let body = vec![
        f.declare(&value, f.int(7)),
        f.declare(&pointer, address(&f, f.values().local(value).unwrap())),
        f.ast()
            .nested_block(
                f.ast()
                    .block(child, vec![f.discard(pointed_read(&f, f.read(&pointer)))])
                    .unwrap(),
            )
            .unwrap(),
        f.discard(pointed_read(&f, f.read(&pointer))),
    ];
    check(&f, body, Ok(()));
}
