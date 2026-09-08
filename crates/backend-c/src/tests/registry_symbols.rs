//! Exact callable and lexical/control owner authentication.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAllocatorSource, CConstness, CFunctionType, CObjectType, CParameterType, CRegistryError,
    CReturnType, CScalarType,
};

fn scalar() -> CObjectType {
    CObjectType::scalar(CScalarType::I32)
}
fn signature() -> CFunctionType {
    CFunctionType::new(
        CReturnType::Void,
        vec![CParameterType::new(scalar()).unwrap()],
    )
}

#[test]
fn function_and_object_registrations_keep_exact_signatures_and_file_origin() {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(&file, key("function"), signature())
        .unwrap();
    let object = registry
        .register_object(&file, key("object"), scalar())
        .unwrap();
    assert_eq!(function.file(), &file);
    assert_eq!(function.signature(), &signature());
    assert_eq!(object.file(), &file);
    assert_eq!(object.ty(), &scalar());
    registry.check_function(&function).unwrap();
    registry.check_object(&object).unwrap();
    assert_eq!(
        registry.register_function(
            &file,
            key("function"),
            CFunctionType::new(CReturnType::Void, vec![])
        ),
        Err(CRegistryError::DuplicateRegistration)
    );
    assert_eq!(
        registry.register_object(&file, key("object"), CObjectType::scalar(CScalarType::Bool)),
        Err(CRegistryError::DuplicateRegistration)
    );
}

#[test]
fn parameter_type_and_index_are_derived_not_supplied_by_the_caller() {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(&file, key("function"), signature())
        .unwrap();
    let other = registry
        .register_function(&file, key("other"), signature())
        .unwrap();
    assert_eq!(
        registry.register_parameter(&function, 1, key("invalid"), CConstness::Unqualified),
        Err(CRegistryError::ParameterIndex)
    );
    let parameter = registry
        .register_parameter(&function, 0, key("value"), CConstness::Const)
        .unwrap();
    assert_eq!(
        parameter.ty(),
        &scalar().with_constness(CConstness::Const).unwrap()
    );
    assert_eq!(parameter.index(), 0);
    registry.check_parameter(&function, &parameter).unwrap();
    assert_eq!(
        registry.check_parameter(&other, &parameter),
        Err(CRegistryError::WrongOwner)
    );
    assert_eq!(
        registry.register_parameter(&function, 0, key("duplicate"), CConstness::Unqualified),
        Err(CRegistryError::DuplicateRegistration)
    );
}

#[test]
fn scopes_locals_and_control_references_cannot_cross_function_owners() {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(&file, key("function"), signature())
        .unwrap();
    let other = registry
        .register_function(&file, key("other"), signature())
        .unwrap();
    let root = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let child = registry
        .register_scope(&function, Some(&root), key("child"))
        .unwrap();
    let other_root = registry.register_scope(&other, None, key("root")).unwrap();
    assert_eq!(child.parent(), Some(&root));
    assert_eq!(
        registry.register_scope(&other, Some(&root), key("crossed")),
        Err(CRegistryError::WrongOwner)
    );
    let local = registry
        .register_local(&child, key("local"), scalar())
        .unwrap();
    assert_eq!(local.scope(), &child);
    assert_eq!(local.ty(), &scalar());
    registry.check_local(&function, &local).unwrap();
    assert_eq!(
        registry.check_local(&other, &local),
        Err(CRegistryError::WrongOwner)
    );
    let loop_ref = registry.register_loop(&child, key("loop")).unwrap();
    let switch = registry.register_switch(&child, key("selection")).unwrap();
    let cleanup = registry
        .register_cleanup_exit(&child, key("cleanup"))
        .unwrap();
    registry.check_loop(&child, &loop_ref).unwrap();
    registry.check_switch(&child, &switch).unwrap();
    registry.check_cleanup_exit(&child, &cleanup).unwrap();
    assert_eq!(
        registry.check_loop(&other_root, &loop_ref),
        Err(CRegistryError::WrongOwner)
    );
    assert_eq!(
        registry.check_switch(&other_root, &switch),
        Err(CRegistryError::WrongOwner)
    );
    assert_eq!(
        registry.check_cleanup_exit(&other_root, &cleanup),
        Err(CRegistryError::WrongOwner)
    );
    assert_eq!(
        registry.register_loop(&root, key("loop")),
        Err(CRegistryError::DuplicateRegistration)
    );
    // Enclosure/visibility/control-flow proof is later; registration is not it.
}

#[test]
fn allocation_id_retains_object_allocator_and_owner_without_claiming_safety() {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(&file, key("function"), signature())
        .unwrap();
    let other = registry
        .register_function(&file, key("other"), signature())
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let other_scope = registry.register_scope(&other, None, key("root")).unwrap();
    let parameter = registry
        .register_parameter(&function, 0, key("allocator"), CConstness::Unqualified)
        .unwrap();
    let local = registry
        .register_local(&scope, key("allocator_local"), scalar())
        .unwrap();
    for (index, source) in [
        CAllocatorSource::Default,
        CAllocatorSource::Parameter(parameter.clone()),
        CAllocatorSource::Local(local),
    ]
    .into_iter()
    .enumerate()
    {
        let allocation = registry
            .register_allocation(
                &scope,
                key(&format!("allocation{index}")),
                scalar(),
                source.clone(),
            )
            .unwrap();
        assert_eq!(allocation.scope(), &scope);
        assert_eq!(allocation.object_type(), &scalar());
        assert_eq!(allocation.allocator(), &source);
        registry.check_allocation(&scope, &allocation).unwrap();
        assert_eq!(
            registry.check_allocation(&other_scope, &allocation),
            Err(CRegistryError::WrongOwner)
        );
    }
    assert_eq!(
        registry.register_allocation(
            &other_scope,
            key("wrong"),
            scalar(),
            CAllocatorSource::Parameter(parameter)
        ),
        Err(CRegistryError::WrongOwner)
    );
    // Actual allocator type/signature, provenance, extent and initialization
    // must be verified at 02B/02D; this registration is deliberately no proof.
}

#[test]
fn matching_foreign_functions_and_scopes_are_not_local_registrations() {
    let (mut home, file) = registry();
    let (mut foreign, foreign_file) = registry();
    let function = home
        .register_function(&file, key("function"), signature())
        .unwrap();
    let other = foreign
        .register_function(&foreign_file, key("function"), signature())
        .unwrap();
    let scope = foreign.register_scope(&other, None, key("root")).unwrap();
    assert_eq!(function.key(), other.key());
    assert_eq!(function.signature(), other.signature());
    assert_eq!(
        home.check_function(&other),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        home.register_scope(&other, None, key("wrong")),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        home.register_local(&scope, key("wrong"), scalar()),
        Err(CRegistryError::CrossRegistry)
    );
}

#[test]
fn registration_has_no_small_parameter_arity_limit() {
    let (mut registry, file) = registry();
    let signature = CFunctionType::new(
        CReturnType::Void,
        vec![CParameterType::new(scalar()).unwrap(); 1024],
    );
    let function = registry
        .register_function(&file, key("many"), signature)
        .unwrap();
    let last = registry
        .register_parameter(&function, 1023, key("last"), CConstness::Unqualified)
        .unwrap();
    assert_eq!(last.index(), 1023);
    assert_eq!(
        registry.register_parameter(&function, 1024, key("outside"), CConstness::Unqualified),
        Err(CRegistryError::ParameterIndex)
    );
}
