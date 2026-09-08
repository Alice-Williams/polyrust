//! Object, pointer, array and exact prototype categories.

use crate::ast::{
    CArrayLength, CConstness, CFunctionType, CObjectType, CObjectTypeKind, CParameterType,
    CPointerTarget, CReturnType, CReturnValue, CScalarType, CTypeError,
};

fn i32_type() -> CObjectType {
    CObjectType::scalar(CScalarType::I32)
}

fn pointer(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}

#[test]
fn every_scalar_is_an_object_parameter_and_unqualified_return() {
    for scalar in CScalarType::ALL {
        let ty = CObjectType::scalar(scalar);
        assert_eq!(ty.kind(), &CObjectTypeKind::Scalar(scalar));
        assert_eq!(ty.constness(), CConstness::Unqualified);
        assert_eq!(CParameterType::new(ty.clone()).unwrap().ty(), &ty);
        assert_eq!(CReturnValue::new(ty.clone()).unwrap().ty(), &ty);
        let qualified = ty.with_constness(CConstness::Const).unwrap();
        assert_eq!(
            CReturnValue::new(qualified),
            Err(CTypeError::QualifiedReturn)
        );
    }
}

#[test]
fn fixed_array_bounds_are_nonzero_without_a_hidden_portable_arity_cap() {
    assert_eq!(CArrayLength::new(0), Err(CTypeError::ZeroArrayLength));
    for length in [1, 2, 127, 1024, u64::MAX] {
        assert_eq!(CArrayLength::new(length).unwrap().get(), length);
    }
    // Large structural bounds are not claims of compiler/object-size capacity.
    // That separate linked-resource check is added before certified generation.
}

#[test]
fn arrays_cannot_be_prototype_parameters_or_return_values() {
    let array = CObjectType::array(i32_type(), CArrayLength::new(3).unwrap()).unwrap();
    assert_eq!(
        CParameterType::new(array.clone()),
        Err(CTypeError::ArrayParameterRequiresPointer)
    );
    assert_eq!(
        CReturnValue::new(array.clone()),
        Err(CTypeError::ArrayReturn)
    );
    assert!(CParameterType::new(pointer(array.clone())).is_ok());
    assert!(CReturnValue::new(pointer(array)).is_ok());
}

#[test]
fn array_qualification_is_applied_to_elements() {
    let array = CObjectType::array(i32_type(), CArrayLength::new(2).unwrap()).unwrap();
    assert_eq!(
        array.clone().with_constness(CConstness::Const),
        Err(CTypeError::ArrayQualifierMustApplyToElement)
    );
    assert_eq!(
        array.clone().with_constness(CConstness::Unqualified),
        Ok(array.clone())
    );
    let qualified = CObjectType::array(
        i32_type().with_constness(CConstness::Const).unwrap(),
        CArrayLength::new(2).unwrap(),
    )
    .unwrap();
    assert_ne!(array, qualified);
    assert_eq!(qualified.constness(), CConstness::Unqualified);
    let CObjectTypeKind::Array { element, .. } = qualified.kind() else {
        panic!("expected structural array");
    };
    assert_eq!(element.constness(), CConstness::Const);
}

#[test]
fn pointer_to_array_and_array_of_pointers_have_distinct_structures() {
    let bound = CArrayLength::new(4).unwrap();
    let pointer_to_array = pointer(CObjectType::array(i32_type(), bound).unwrap());
    let array_of_pointers = CObjectType::array(pointer(i32_type()), bound).unwrap();
    assert_ne!(pointer_to_array, array_of_pointers);
    assert!(!pointer_to_array.is_array());
    assert!(array_of_pointers.is_array());
    assert!(CReturnValue::new(pointer_to_array).is_ok());
    assert_eq!(
        CReturnValue::new(array_of_pointers),
        Err(CTypeError::ArrayReturn)
    );
}

#[test]
fn function_pointers_retain_their_full_prototype() {
    let function = CFunctionType::new(
        CReturnType::Value(CReturnValue::new(i32_type()).unwrap()),
        vec![CParameterType::new(pointer(i32_type())).unwrap()],
    );
    let function_pointer =
        CObjectType::pointer(CPointerTarget::Function(Box::new(function.clone())));
    let pointers =
        CObjectType::array(function_pointer.clone(), CArrayLength::new(3).unwrap()).unwrap();
    assert!(pointers.is_array());
    let CObjectTypeKind::Pointer(CPointerTarget::Function(actual)) = function_pointer.kind() else {
        panic!("expected typed function pointer");
    };
    assert_eq!(actual.as_ref(), &function);
    let returning_pointer = CFunctionType::new(
        CReturnType::Value(CReturnValue::new(function_pointer).unwrap()),
        vec![],
    );
    assert!(returning_pointer.parameters().is_empty());
    assert_ne!(returning_pointer, function);
}

#[test]
fn prototype_normalization_removes_only_top_level_qualifiers() {
    let qualified_scalar = i32_type().with_constness(CConstness::Const).unwrap();
    assert_eq!(
        CParameterType::new(qualified_scalar.clone()),
        CParameterType::new(i32_type())
    );
    let pointee_const = pointer(qualified_scalar);
    let pointer_const = pointer(i32_type())
        .with_constness(CConstness::Const)
        .unwrap();
    assert_eq!(
        CParameterType::new(pointer_const),
        CParameterType::new(pointer(i32_type()))
    );
    assert_ne!(
        CParameterType::new(pointee_const.clone()),
        CParameterType::new(pointer(i32_type()))
    );
    assert!(CReturnValue::new(pointee_const).is_ok());
}

#[test]
fn void_object_and_function_pointer_targets_remain_different_categories() {
    let mutable_void = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
    let const_void = CObjectType::pointer(CPointerTarget::Void(CConstness::Const));
    let function = CObjectType::pointer(CPointerTarget::Function(Box::new(CFunctionType::new(
        CReturnType::Void,
        vec![],
    ))));
    assert_ne!(mutable_void, const_void);
    assert_ne!(mutable_void, function);
    assert_ne!(mutable_void, pointer(i32_type()));
    for ty in [mutable_void, const_void, function] {
        assert!(CParameterType::new(ty.clone()).is_ok());
        assert!(CReturnValue::new(ty).is_ok());
    }
}

#[test]
fn exact_prototypes_support_zero_and_many_parameters() {
    let empty = CFunctionType::new(CReturnType::Void, vec![]);
    assert!(empty.parameters().is_empty());
    assert_eq!(empty.return_type(), &CReturnType::Void);
    let many = CFunctionType::new(
        CReturnType::Void,
        vec![CParameterType::new(i32_type()).unwrap(); 1024],
    );
    assert_eq!(many.parameters().len(), 1024);
}
