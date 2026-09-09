//! Static startup is not a callable entry contract or an allocation witness.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, storage_fixture::*, *};

#[test]
fn imported_stream_pointer_values_are_unproved_even_when_only_discarded() {
    for constant in [
        CKnownConstant::StandardInput,
        CKnownConstant::StandardOutput,
        CKnownConstant::StandardError,
        CKnownConstant::IntMax,
    ] {
        let f = Fixture::new(&[]);
        let source = f.source(vec![f.discard(f.values().known_constant(constant))]);
        f.registry
            .check_numeric_flow(std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if constant == CKnownConstant::IntMax {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedStorage)
            },
            "{constant:?}"
        );
    }
}

#[test]
fn global_store_rejects_automatic_but_accepts_static_backing_storage() {
    for automatic in [false, true] {
        let mut f = Fixture::new(&[]);
        let ty = CObjectType::scalar(CScalarType::Int);
        let static_value = f
            .registry
            .register_object(&f.file, key("static_value"), ty.clone())
            .unwrap();
        let output = f
            .registry
            .register_object(&f.file, key("output"), pointer_type(ty))
            .unwrap();
        let value = f.local(CScalarType::Int, "value");
        let selected = if automatic {
            f.values().local(value.clone()).unwrap()
        } else {
            f.values().global(static_value.clone()).unwrap()
        };
        let mut body = vec![
            f.declare(&value, f.int(1)),
            f.ast()
                .assign(
                    f.values().global(output.clone()).unwrap(),
                    address(&f, selected),
                )
                .unwrap(),
            f.discard(pointed_read(
                &f,
                f.values()
                    .read(f.values().global(output.clone()).unwrap())
                    .unwrap(),
            )),
        ];
        if automatic {
            body.pop();
        }
        let mut source = f.source(body);
        let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
        for object in [static_value, output] {
            source.items.insert(
                0,
                CFileItem::Definition(
                    declarations
                        .object_definition(
                            object.clone(),
                            CLinkage::Internal,
                            f.values().zero_initializer(object.ty().clone()).unwrap(),
                        )
                        .unwrap(),
                ),
            );
        }
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if automatic {
                Err(CSafetyError::AutomaticAddressEscape)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn imported_pointer_parameter_cannot_be_read_without_a_storage_contract() {
    let mut f = Fixture::new(&[]);
    let ty = pointer_type(CObjectType::scalar(CScalarType::Int));
    let function = f
        .registry
        .register_function(
            &f.file,
            key("imported"),
            CFunctionType::new(CReturnType::Void, vec![CParameterType::new(ty).unwrap()]),
        )
        .unwrap();
    let scope = f
        .registry
        .register_scope(&function, None, key("entry"))
        .unwrap();
    let parameter = f
        .registry
        .register_parameter(&function, 0, key("pointer"), CConstness::Unqualified)
        .unwrap();
    let read = f
        .values()
        .read(f.values().parameter(parameter.clone()).unwrap())
        .unwrap();
    let statements = CStatements::new(&f.registry, function.clone()).unwrap();
    let body = statements
        .block(scope, vec![statements.discard(read).unwrap()])
        .unwrap();
    let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
    let imported = declarations
        .source_file(vec![CFileItem::Definition(
            declarations
                .function_definition(function, CLinkage::External, vec![parameter], body)
                .unwrap(),
        )])
        .unwrap();
    let mut source = f.source(vec![]);
    source.items.extend(imported.items);
    f.registry
        .check_numeric_flow(std::slice::from_ref(&source))
        .unwrap();
    assert_eq!(
        f.registry.check_storage_paths(&[source]),
        Err(CSafetyError::UnprovedStorage)
    );
}

#[test]
fn mutable_global_pointer_initializer_is_not_assumed_at_function_entry() {
    for dereference in [false, true] {
        let mut f = Fixture::new(&[]);
        let value = f
            .registry
            .register_object(&f.file, key("value"), CObjectType::scalar(CScalarType::Int))
            .unwrap();
        let pointer = f
            .registry
            .register_object(&f.file, key("pointer"), pointer_type(value.ty().clone()))
            .unwrap();
        let read = f
            .values()
            .read(f.values().global(pointer.clone()).unwrap())
            .unwrap();
        let mut source = f.source(vec![f.discard(if dereference {
            pointed_read(&f, read)
        } else {
            read
        })]);
        let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
        source.items.insert(
            0,
            CFileItem::Definition(
                declarations
                    .object_definition(
                        pointer,
                        CLinkage::Internal,
                        f.values()
                            .expression_initializer(address(
                                &f,
                                f.values().global(value.clone()).unwrap(),
                            ))
                            .unwrap(),
                    )
                    .unwrap(),
            ),
        );
        source.items.insert(
            0,
            CFileItem::Definition(
                declarations
                    .object_definition(
                        value.clone(),
                        CLinkage::Internal,
                        f.values().zero_initializer(value.ty().clone()).unwrap(),
                    )
                    .unwrap(),
            ),
        );
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            Err(CSafetyError::UnprovedStorage)
        );
    }
}

#[test]
fn allocation_registration_and_void_conversion_cannot_invent_allocation_evidence() {
    for wrong_type in [false, true] {
        let mut f = Fixture::new(&[]);
        let value = f.local(CScalarType::Int, "value");
        let allocation = f
            .registry
            .register_allocation(
                &f.scope,
                key("allocation"),
                CObjectType::scalar(if wrong_type {
                    CScalarType::I64
                } else {
                    CScalarType::Int
                }),
                CAllocatorSource::Default,
            )
            .unwrap();
        let erased = f
            .values()
            .object_to_void(
                CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified)),
                address(&f, f.values().local(value.clone()).unwrap()),
            )
            .unwrap();
        let restored = f.values().allocation_restore(allocation, erased).unwrap();
        check(
            &f,
            vec![f.declare(&value, f.int(1)), f.discard(restored)],
            Err(CSafetyError::UnprovedAllocation),
        );
    }
}

#[test]
fn registered_generated_call_signature_does_not_establish_storage_effects() {
    let mut f = Fixture::new(&[]);
    let callee = f
        .registry
        .register_function(
            &f.file,
            key("callee"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let callee_scope = f
        .registry
        .register_scope(&callee, None, key("callee_scope"))
        .unwrap();
    let call = f
        .values()
        .call_effect(f.values().direct(callee.clone()).unwrap(), vec![])
        .unwrap();
    let mut source = f.source(vec![f.ast().evaluate(call).unwrap()]);
    source.items.insert(
        0,
        CFileItem::Definition(
            CDeclarations::new(&f.registry, f.file.clone())
                .unwrap()
                .function_definition(
                    callee.clone(),
                    CLinkage::External,
                    vec![],
                    CStatements::new(&f.registry, callee)
                        .unwrap()
                        .block(callee_scope, vec![])
                        .unwrap(),
                )
                .unwrap(),
        ),
    );
    assert_eq!(
        f.registry.check_storage_paths(&[source]),
        Err(CSafetyError::UnprovedStorageCall)
    );
}
