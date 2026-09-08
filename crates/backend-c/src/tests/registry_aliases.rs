//! Alias categories and qualifiers cannot bypass private prototype witnesses.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CArrayLength, CConstness, CFunctionType, CObjectType, CObjectTypeKind, CParameterType,
    CPointerTarget, CRegistryError, CReturnType, CReturnValue, CScalarType, CTypeError,
};

fn scalar() -> CObjectType {
    CObjectType::scalar(CScalarType::I32)
}
fn pointer(value: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(value)))
}

#[test]
fn nested_alias_arrays_cannot_be_parameters_returns_or_top_qualified() {
    let (mut registry, file) = registry();
    let array = CObjectType::array(
        scalar().with_constness(CConstness::Const).unwrap(),
        CArrayLength::new(4).unwrap(),
    )
    .unwrap();
    let first = registry
        .register_typedef(&file, key("Array"), array.clone())
        .unwrap();
    let nested = registry
        .register_typedef(&file, key("Nested"), CObjectType::typedef(first))
        .unwrap();
    let ty = CObjectType::typedef(nested);
    assert!(ty.is_array());
    assert_eq!(ty.canonical(), array);
    assert_eq!(
        CParameterType::new(ty.clone()),
        Err(CTypeError::ArrayParameterRequiresPointer)
    );
    assert_eq!(CReturnValue::new(ty.clone()), Err(CTypeError::ArrayReturn));
    assert_eq!(
        ty.clone().with_constness(CConstness::Const),
        Err(CTypeError::ArrayQualifierMustApplyToElement)
    );
    let adjusted = CParameterType::new(pointer(ty)).unwrap();
    assert_eq!(adjusted.ty(), &pointer(array));
}

#[test]
fn aliases_cannot_hide_return_const_or_erase_pointee_const() {
    let (mut registry, file) = registry();
    let qualified = scalar().with_constness(CConstness::Const).unwrap();
    let alias = registry
        .register_typedef(&file, key("Qualified"), qualified.clone())
        .unwrap();
    let nested = registry
        .register_typedef(&file, key("Nested"), CObjectType::typedef(alias))
        .unwrap();
    let ty = CObjectType::typedef(nested);
    assert_eq!(
        CReturnValue::new(ty.clone()),
        Err(CTypeError::QualifiedReturn)
    );
    let parameter = CParameterType::new(ty.clone()).unwrap();
    assert_eq!(parameter.ty(), &scalar());
    assert_eq!(parameter.declared_type(), &ty);
    assert_eq!(
        CParameterType::new(pointer(ty.clone())).unwrap().ty(),
        &pointer(qualified.clone())
    );
    assert_eq!(
        CReturnValue::new(pointer(ty)).unwrap().ty(),
        &pointer(qualified)
    );
}

#[test]
fn canonical_alias_normalization_does_not_unfold_nominal_records() {
    let (mut registry, file) = registry();
    let nominal = registry.declare_struct(&file, key("Record")).unwrap();
    let alias = registry
        .register_typedef(&file, key("Alias"), CObjectType::structure(nominal.clone()))
        .unwrap();
    let ty = CObjectType::typedef(alias);
    assert_eq!(ty.canonical(), CObjectType::structure(nominal));
    assert!(matches!(ty.kind(), CObjectTypeKind::Typedef(_)));
    assert!(CReturnValue::new(ty).is_ok());
}

#[test]
fn normalized_scalar_alias_still_retains_cross_registry_authentication() {
    let (mut home, home_file) = registry();
    let (mut foreign, foreign_file) = registry();
    let alias = foreign
        .register_typedef(&foreign_file, key("Alias"), scalar())
        .unwrap();
    let ty = CObjectType::typedef(alias);
    let parameter = CParameterType::new(ty.clone()).unwrap();
    let result = CReturnValue::new(ty).unwrap();
    assert_eq!(parameter.ty(), &scalar());
    assert_eq!(result.ty(), &scalar());
    let parameter_signature = CFunctionType::new(CReturnType::Void, vec![parameter]);
    let result_signature = CFunctionType::new(CReturnType::Value(result), vec![]);
    for signature in [parameter_signature, result_signature] {
        assert_eq!(
            home.check_signature(&signature),
            Err(CRegistryError::CrossRegistry)
        );
        let callback = CObjectType::pointer(CPointerTarget::Function(Box::new(signature)));
        assert_eq!(
            home.register_typedef(&home_file, key("Callback"), callback),
            Err(CRegistryError::CrossRegistry)
        );
    }
}

#[test]
fn alias_chain_construction_is_immutable_acyclic_and_normalizes_iteratively() {
    let (mut registry, file) = registry();
    let mut ty = scalar();
    for depth in 0..128 {
        let alias = registry
            .register_typedef(&file, key(&format!("Alias{depth}")), ty)
            .unwrap();
        ty = CObjectType::typedef(alias);
    }
    registry.check_type(&ty).unwrap();
    assert_eq!(ty.canonical(), scalar());
    assert_eq!(CParameterType::new(ty).unwrap().ty(), &scalar());
    // No alias-forward-declaration or target mutation API can introduce a cycle.
}
