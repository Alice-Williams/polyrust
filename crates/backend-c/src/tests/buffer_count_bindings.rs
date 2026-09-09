//! Typed count references retain real immutable declarations, never capacity claims.
use super::{
    allocation_fixture::*, contextual_reconstruction::key, heap_fixture::*,
    numeric_fixture::Fixture, *,
};

#[test]
fn count_registration_and_checked_binding_preserve_the_exact_immutable_size_local() {
    let mut f = Fixture::new(&[]);
    let count = f
        .registry
        .register_buffer_count(&f.scope, key("count"))
        .unwrap();
    assert_eq!(count.local().scope(), &f.scope);
    assert_eq!(
        count.local().ty(),
        &CObjectType::scalar(CScalarType::Size)
            .with_constness(CConstness::Const)
            .unwrap()
    );
    assert_eq!(
        f.registry.buffer_count_reference(count.local()),
        Ok(count.clone())
    );
    assert!(f.ast().declare(count.local().clone(), None).is_err());
    assert!(
        f.ast()
            .assign(f.values().local(count.local().clone()).unwrap(), f.size(3))
            .is_err()
    );
    let source = f.source(vec![f.declare(count.local(), f.size(2))]);
    assert_eq!(f.registry.check_storage_paths(&[source]), Ok(()));
}

#[test]
fn arbitrary_scalar_locals_cannot_be_relabelled_as_immutable_size_counts() {
    for scalar in [
        CScalarType::Bool,
        CScalarType::Int,
        CScalarType::U64,
        CScalarType::Size,
        CScalarType::F64,
    ] {
        for qualifier in [CConstness::Unqualified, CConstness::Const] {
            let mut f = Fixture::new(&[]);
            let local = f
                .registry
                .register_local(
                    &f.scope,
                    key("count"),
                    CObjectType::scalar(scalar)
                        .with_constness(qualifier)
                        .unwrap(),
                )
                .unwrap();
            let result = f.registry.buffer_count_reference(&local);
            if scalar == CScalarType::Size && qualifier == CConstness::Const {
                assert_eq!(result.unwrap().local(), &local);
            } else {
                assert_eq!(result, Err(CRegistryError::InvalidBufferCount));
            }
        }
    }
}

#[test]
fn count_references_authenticate_registry_and_buffer_owner() {
    let mut f = Fixture::new(&[]);
    let mut foreign = Fixture::new(&[]);
    let count = foreign
        .registry
        .register_buffer_count(&foreign.scope, key("count"))
        .unwrap();
    assert_eq!(
        f.registry.buffer_count_reference(count.local()),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        f.registry.register_buffer_allocation(
            &f.scope,
            key("buffer"),
            CObjectType::scalar(CScalarType::Int),
            count,
            CAllocatorSource::Default
        ),
        Err(CRegistryError::CrossRegistry)
    );
    let count = f
        .registry
        .register_buffer_count(&f.scope, key("count"))
        .unwrap();
    let other = f
        .registry
        .register_function(
            &f.file,
            key("other"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let scope = f
        .registry
        .register_scope(&other, None, key("other_scope"))
        .unwrap();
    assert_eq!(
        f.registry.register_buffer_allocation(
            &scope,
            key("buffer"),
            CObjectType::scalar(CScalarType::Int),
            count,
            CAllocatorSource::Default
        ),
        Err(CRegistryError::WrongOwner)
    );
}

#[test]
fn buffer_shape_retains_count_and_element_but_cannot_claim_fixed_storage() {
    let mut f = Fixture::new(&[]);
    let count = f
        .registry
        .register_buffer_count(&f.scope, key("count"))
        .unwrap();
    let element = CObjectType::scalar(CScalarType::Int);
    let buffer = f
        .registry
        .register_buffer_allocation(
            &f.scope,
            key("buffer"),
            element.clone(),
            count.clone(),
            CAllocatorSource::Default,
        )
        .unwrap();
    assert_eq!(buffer.shape(), &CAllocationShape::Elements(count.clone()));
    assert_eq!(buffer.object_type(), &element);
    let raw = raw(&mut f, "raw");
    let live = require_live(&mut f, &raw, vec![]);
    let value = restore(&f, &buffer, f.read(&raw));
    assert_eq!(
        value.ty(),
        &CObjectType::pointer(CPointerTarget::Object(Box::new(element)))
    );
    let source = f.source(vec![
        f.declare(count.local(), f.size(2)),
        f.declare(&raw, allocate(&f, f.size(8))),
        live,
        f.discard(value),
        release(&f, f.read(&raw)),
    ]);
    f.registry
        .check_lexical_structure(std::slice::from_ref(&source))
        .unwrap();
    assert_eq!(
        f.registry.check_storage_paths(&[source]),
        Err(CSafetyError::UnprovedAllocationSize)
    );
}

#[test]
fn count_metadata_cannot_bypass_actual_declaration_dominance() {
    for declared in [false, true] {
        let mut f = Fixture::new(&[]);
        let count = f
            .registry
            .register_buffer_count(&f.scope, key("count"))
            .unwrap();
        let buffer = f
            .registry
            .register_buffer_allocation(
                &f.scope,
                key("buffer"),
                CObjectType::scalar(CScalarType::Int),
                count.clone(),
                CAllocatorSource::Default,
            )
            .unwrap();
        let raw = raw(&mut f, "raw");
        let declaration = f.declare(count.local(), f.size(2));
        let mut body = vec![f.declare(&raw, null(&f, &raw))];
        if declared {
            body.push(declaration.clone());
        }
        body.push(f.discard(restore(&f, &buffer, f.read(&raw))));
        if !declared {
            body.push(declaration);
        }
        assert_eq!(
            f.registry.check_lexical_structure(&[f.source(body)]),
            if declared {
                Ok(())
            } else {
                Err(CContextError::InvisibleBinding)
            }
        );
    }
}

#[test]
fn actual_matching_product_establishes_empty_element_storage() {
    let mut f = Fixture::new(&[]);
    let count = f
        .registry
        .register_buffer_count(&f.scope, key("count"))
        .unwrap();
    let descriptor = f
        .registry
        .register_buffer_allocation(
            &f.scope,
            key("buffer"),
            CObjectType::scalar(CScalarType::Int),
            count.clone(),
            CAllocatorSource::Default,
        )
        .unwrap();
    let raw = raw(&mut f, "raw");
    let live = require_live(&mut f, &raw, vec![]);
    let product = f.binary(CBinaryOperator::Multiply, f.read(count.local()), f.size(4));
    let source = f.source(vec![
        f.declare(count.local(), f.size(2)),
        f.declare(&raw, allocate(&f, product)),
        live,
        f.discard(restore(&f, &descriptor, f.read(&raw))),
        release(&f, f.read(&raw)),
    ]);
    assert_eq!(f.registry.check_storage_paths(&[source]), Ok(()));
}
