//! C compatibility does not erase registered origins or nested qualifiers.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CArrayLength, CConstness, CFunctionType, CKnownObject, CObjectType, CParameterType,
    CPointerTarget, CRegistryError, CReturnType, CReturnValue, CScalarType,
};

fn scalar(ty: CScalarType) -> CObjectType {
    CObjectType::scalar(ty)
}
fn pointer(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}
fn signature(ty: CObjectType, count: usize) -> CFunctionType {
    CFunctionType::new(
        CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
        vec![CParameterType::new(ty).unwrap(); count],
    )
}

#[test]
fn scalar_matching_uses_abi_identity_without_implicit_numeric_conversion() {
    let (registry, _) = registry();
    for left in CScalarType::ALL {
        for right in CScalarType::ALL {
            assert_eq!(
                registry.types_match(&scalar(left), &scalar(right)),
                Ok(left.is_compatible_with(right))
            );
        }
    }
}

#[test]
fn aliases_match_only_after_authentication_and_nested_qualifiers_remain_exact() {
    let (mut owner, file) = registry();
    let alias = owner
        .register_typedef(&file, key("Count"), scalar(CScalarType::Size))
        .unwrap();
    let ty = CObjectType::typedef(alias);
    assert_eq!(owner.types_match(&ty, &scalar(CScalarType::U64)), Ok(true));
    let qualified = ty.clone().with_constness(CConstness::Const).unwrap();
    assert_eq!(owner.types_match(&ty, &qualified), Ok(false));
    assert_eq!(
        owner.types_match(&pointer(ty.clone()), &pointer(qualified.clone())),
        Ok(false)
    );
    assert_eq!(
        owner.types_match(&pointer(pointer(ty.clone())), &pointer(pointer(qualified))),
        Ok(false)
    );
    let (foreign, _) = registry();
    assert_eq!(
        foreign.types_match(&ty, &scalar(CScalarType::U64)),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        foreign.types_match(&scalar(CScalarType::I32), &ty),
        Err(CRegistryError::CrossRegistry)
    );
}

#[test]
fn arrays_keep_bounds_nominals_keep_identity_and_known_types_keep_ownership() {
    let (mut registry, file) = registry();
    let first = CObjectType::structure(registry.declare_struct(&file, key("First")).unwrap());
    let second = CObjectType::structure(registry.declare_struct(&file, key("Second")).unwrap());
    assert_eq!(registry.types_match(&first, &first), Ok(true));
    assert_eq!(registry.types_match(&first, &second), Ok(false));
    let union = CObjectType::union(registry.declare_union(&file, key("Union")).unwrap());
    let enumeration = CObjectType::enumeration(registry.declare_enum(&file, key("Enum")).unwrap());
    let known = CObjectType::known(CKnownObject::MaxAlign);
    for ty in [first.clone(), second, union, enumeration, known] {
        assert_eq!(registry.types_match(&ty, &ty), Ok(true));
        assert_eq!(
            registry.types_match(&ty, &scalar(CScalarType::Int)),
            Ok(false)
        );
    }
    let a = CObjectType::array(first.clone(), CArrayLength::new(1).unwrap()).unwrap();
    let b = CObjectType::array(first.clone(), CArrayLength::new(2).unwrap()).unwrap();
    assert_eq!(registry.types_match(&a, &a), Ok(true));
    assert_eq!(registry.types_match(&a, &b), Ok(false));
    assert_eq!(registry.types_match(&a, &pointer(first)), Ok(false));
    assert_eq!(
        registry.types_match(
            &CObjectType::known(CKnownObject::File),
            &CObjectType::known(CKnownObject::MaxAlign)
        ),
        Ok(false)
    );
}

#[test]
fn callback_matching_uses_actual_types_but_does_not_authenticate_contracts() {
    let (mut owner, file) = registry();
    let alias = owner
        .register_typedef(&file, key("Count"), scalar(CScalarType::I32))
        .unwrap();
    let a = signature(CObjectType::typedef(alias), 100);
    let b = signature(scalar(CScalarType::Int), 100);
    assert_eq!(owner.signatures_match(&a, &b), Ok(true));
    assert_ne!(a, b); // declared alias provenance is still part of the Rust data.
    assert_eq!(
        owner.signatures_match(&a, &signature(scalar(CScalarType::Int), 99)),
        Ok(false)
    );
    assert_eq!(
        owner.signatures_match(&a, &signature(scalar(CScalarType::U32), 100)),
        Ok(false)
    );
    let void = CFunctionType::new(CReturnType::Void, b.parameters().to_vec());
    assert_eq!(owner.signatures_match(&b, &void), Ok(false));
    assert_eq!(owner.signatures_match(&void, &void), Ok(true));
    let pointer_a = CObjectType::pointer(CPointerTarget::Function(Box::new(a)));
    let pointer_b = CObjectType::pointer(CPointerTarget::Function(Box::new(b)));
    assert_eq!(owner.types_match(&pointer_a, &pointer_b), Ok(true));
    let (foreign, _) = registry();
    assert_eq!(
        foreign.types_match(&pointer_a, &pointer_b),
        Err(CRegistryError::CrossRegistry)
    );
    let void_pointer = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
    assert_eq!(owner.types_match(&pointer_a, &void_pointer), Ok(false));
    assert_eq!(
        owner.types_match(&void_pointer, &pointer(scalar(CScalarType::I32))),
        Ok(false)
    );
    assert_eq!(
        owner.types_match(
            &void_pointer,
            &CObjectType::pointer(CPointerTarget::Void(CConstness::Const))
        ),
        Ok(false)
    );
}
