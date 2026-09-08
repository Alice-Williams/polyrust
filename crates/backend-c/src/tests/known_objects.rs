//! Library identities and opaque storage restrictions survive aliases.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CAllocatorSource, CArrayLength, CConstness, CFunctionType, CKnownObject,
    CObjectType, CParameterType, CPointerTarget, CRegistryError, CReturnType, CReturnValue,
    CScalarType, CTypeError,
};

fn pointer(value: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(value)))
}

#[test]
fn opaque_file_values_cannot_be_parameters_returns_or_array_elements() {
    let (mut registry, file) = registry();
    let known = CObjectType::known(CKnownObject::File);
    let alias = registry
        .register_typedef(&file, key("Stream"), known.clone())
        .unwrap();
    let nested = registry
        .register_typedef(&file, key("StreamAlias"), CObjectType::typedef(alias))
        .unwrap();
    let error = CTypeError::KnownObjectRequiresPointer(CKnownObject::File);
    for ty in [known, CObjectType::typedef(nested)] {
        for qualifier in [CConstness::Unqualified, CConstness::Const] {
            let ty = ty.clone().with_constness(qualifier).unwrap();
            assert_eq!(CParameterType::new(ty.clone()), Err(error));
            assert_eq!(CReturnValue::new(ty.clone()), Err(error));
            assert_eq!(
                CObjectType::array(ty.clone(), CArrayLength::new(1).unwrap()),
                Err(error)
            );
            let borrowed = pointer(ty);
            assert!(CParameterType::new(borrowed.clone()).is_ok());
            assert!(CReturnValue::new(borrowed.clone()).is_ok());
            assert!(CObjectType::array(borrowed, CArrayLength::new(2).unwrap()).is_ok());
        }
    }
}

#[test]
fn storage_registrations_reject_opaque_values_but_accept_their_pointers() {
    let (mut registry, file) = registry();
    let owner = CAggregateRef::Struct(registry.declare_struct(&file, key("Holder")).unwrap());
    let function = registry
        .register_function(
            &file,
            key("run"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    let ty = CObjectType::known(CKnownObject::File);
    let alias = registry
        .register_typedef(&file, key("Stream"), ty.clone())
        .unwrap();
    let error = CRegistryError::InvalidObjectType(CTypeError::KnownObjectRequiresPointer(
        CKnownObject::File,
    ));
    for ty in [ty, CObjectType::typedef(alias)] {
        assert_eq!(
            registry.register_object(&file, key("global"), ty.clone()),
            Err(error)
        );
        assert_eq!(
            registry.register_local(&scope, key("local"), ty.clone()),
            Err(error)
        );
        assert_eq!(
            registry.register_member(&owner, key("stream"), ty.clone()),
            Err(error)
        );
        assert_eq!(
            registry.register_allocation(&scope, key("allocation"), ty, CAllocatorSource::Default),
            Err(error)
        );
    }
    let borrowed = pointer(CObjectType::known(CKnownObject::File));
    // Failed registrations do not consume the valid declaration keys.
    assert!(
        registry
            .register_object(&file, key("global"), borrowed.clone())
            .is_ok()
    );
    assert!(
        registry
            .register_local(&scope, key("local"), borrowed.clone())
            .is_ok()
    );
    assert!(
        registry
            .register_member(&owner, key("stream"), borrowed.clone())
            .is_ok()
    );
    assert!(
        registry
            .register_allocation(
                &scope,
                key("allocation"),
                borrowed,
                CAllocatorSource::Default
            )
            .is_ok()
    );
}

#[test]
fn complete_max_align_is_storable_and_not_a_generated_name_alias() {
    let (mut registry, file) = registry();
    let ty = CObjectType::known(CKnownObject::MaxAlign);
    assert_eq!(ty.canonical(), ty);
    assert!(CParameterType::new(ty.clone()).is_ok());
    assert!(CReturnValue::new(ty.clone()).is_ok());
    assert!(CObjectType::array(ty.clone(), CArrayLength::new(2).unwrap()).is_ok());
    assert!(
        registry
            .register_object(&file, key("alignment"), ty.clone())
            .is_ok()
    );
    let generated = registry
        .register_typedef(
            &file,
            key("max_align_t"),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    assert_ne!(CObjectType::typedef(generated).canonical(), ty);
    let stream = registry.declare_struct(&file, key("FILE")).unwrap();
    assert_ne!(
        CObjectType::structure(stream),
        CObjectType::known(CKnownObject::File)
    );
}

#[test]
fn borrowed_alias_authentication_and_parameter_provenance_are_retained() {
    let (mut first, file) = registry();
    let alias = first
        .register_typedef(&file, key("Stream"), CObjectType::known(CKnownObject::File))
        .unwrap();
    let declared = pointer(CObjectType::typedef(alias));
    let signature = CFunctionType::new(
        CReturnType::Void,
        vec![CParameterType::new(declared.clone()).unwrap()],
    );
    let (second, _) = registry();
    assert_eq!(
        second.check_signature(&signature),
        Err(CRegistryError::CrossRegistry)
    );
    let function = first
        .register_function(&file, key("read_stream"), signature)
        .unwrap();
    let parameter = first
        .register_parameter(&function, 0, key("stream"), CConstness::Const)
        .unwrap();
    assert_eq!(
        parameter.function().signature().parameters()[0].declared_type(),
        &declared
    );
    assert_eq!(parameter.ty().constness(), CConstness::Const);
    assert_eq!(
        parameter.ty().canonical(),
        pointer(CObjectType::known(CKnownObject::File))
            .with_constness(CConstness::Const)
            .unwrap()
    );
}
