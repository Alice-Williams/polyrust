//! Value trees retain actual C promotions, reference origins and shape checks.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CBinaryOperator as B, CExpressionError as E, CExpressions, CFunctionType, CKnownObject,
    CLiteral, CObjectType, CRegistryError, CReturnType, CScalarType as T, CSignedLiteral as S,
    CUnaryOperator as U, CValueKind,
};

#[test]
fn nested_mathematics_preserves_the_tree_and_actual_promotions() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let one = ast.literal(CLiteral::Signed(S::I8(1))).unwrap();
    let two = ast.literal(CLiteral::Signed(S::I16(2))).unwrap();
    let sum = ast.binary(B::Add, one.clone(), two.clone()).unwrap();
    let difference = ast.binary(B::Subtract, two, one.clone()).unwrap();
    let product = ast
        .binary(B::Multiply, sum.clone(), difference.clone())
        .unwrap();
    assert_eq!(product.ty(), &CObjectType::scalar(T::Int));
    assert_eq!(
        product.kind(),
        &CValueKind::Binary {
            operator: B::Multiply,
            left: Box::new(sum),
            right: Box::new(difference)
        }
    );
    let negated = ast.unary(U::Negate, one.clone()).unwrap();
    assert_eq!(negated.ty(), &CObjectType::scalar(T::Int));
    assert!(matches!(ast.unary(U::LogicalNot, one), Err(E::Operator(_))));
}

#[test]
fn conditions_require_bool_and_conditional_values_use_c_promotions() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let truth = ast.literal(CLiteral::Bool(true)).unwrap();
    let one = ast.literal(CLiteral::Signed(S::I8(1))).unwrap();
    let comparison = ast.binary(B::Less, one.clone(), one.clone()).unwrap();
    assert_eq!(comparison.ty(), &CObjectType::scalar(T::Int));
    assert_eq!(
        ast.conditional(comparison.clone(), one.clone(), one.clone()),
        Err(E::ExpectedBool)
    );
    let condition = ast.numeric_conversion(T::Bool, comparison).unwrap();
    let result = ast.conditional(condition, one.clone(), one).unwrap();
    assert_eq!(result.ty(), &CObjectType::scalar(T::Int));
    let bool_result = ast
        .conditional(truth.clone(), truth.clone(), truth.clone())
        .unwrap();
    assert_eq!(bool_result.ty(), &CObjectType::scalar(T::Int));
    let long = ast.literal(CLiteral::Signed(S::I64(1))).unwrap();
    assert_eq!(
        ast.conditional(truth.clone(), truth, long),
        Err(E::TypeMismatch)
    );
}

#[test]
fn foreign_values_and_places_cannot_hide_behind_scalar_type_equality() {
    let (first, _) = registry();
    let (mut second, file) = registry();
    let global = second
        .register_object(&file, key("value"), CObjectType::scalar(T::I32))
        .unwrap();
    let a = CExpressions::new(&first);
    let b = CExpressions::new(&second);
    let foreign = b.literal(CLiteral::Signed(S::I32(1))).unwrap();
    let own = a.literal(CLiteral::Signed(S::I32(1))).unwrap();
    let error = E::Registry(CRegistryError::CrossRegistry);
    assert_eq!(a.binary(B::Add, own, foreign.clone()), Err(error));
    assert_eq!(a.numeric_conversion(T::I64, foreign), Err(error));
    assert_eq!(a.read(b.global(global).unwrap()), Err(error));
}

#[test]
fn enum_constants_are_int_but_native_enum_storage_uses_its_definition() {
    let (mut registry, file) = registry();
    let signed = registry.declare_enum(&file, key("Signed")).unwrap();
    let unsigned = registry.declare_enum(&file, key("Unsigned")).unwrap();
    let incomplete = registry.declare_enum(&file, key("Incomplete")).unwrap();
    let negative = registry
        .register_enumerator(&signed, key("Negative"), -1)
        .unwrap();
    let positive = registry
        .register_enumerator(&unsigned, key("Positive"), i32::MAX)
        .unwrap();
    registry
        .define_enum(&signed, vec![negative.clone()])
        .unwrap();
    registry
        .define_enum(&unsigned, vec![positive.clone()])
        .unwrap();
    let globals = [signed, unsigned, incomplete]
        .into_iter()
        .enumerate()
        .map(|(index, ty)| {
            registry
                .register_object(
                    &file,
                    key(&format!("value{index}")),
                    CObjectType::enumeration(ty),
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    let ast = CExpressions::new(&registry);
    for value in [negative, positive] {
        assert_eq!(
            ast.enumerator(value).unwrap().ty(),
            &CObjectType::scalar(T::Int)
        );
    }
    for (global, result) in
        globals
            .into_iter()
            .zip([Ok(T::Int), Ok(T::U32), Err(E::IncompleteEnum)])
    {
        let value = ast.read(ast.global(global).unwrap()).unwrap();
        assert_eq!(
            ast.unary(U::BitNot, value).map(|value| value.ty().clone()),
            result.map(CObjectType::scalar)
        );
    }
}

#[test]
fn sizeof_alignof_and_function_addresses_authenticate_types_without_certifying_completeness() {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(
            &file,
            key("run"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let address = ast.function_address(function.clone()).unwrap();
    assert_eq!(address.kind(), &CValueKind::FunctionAddress(function));
    assert_eq!(ast.unary(U::Negate, address), Err(E::ExpectedArithmetic));
    for ty in [
        CObjectType::scalar(T::F64),
        CObjectType::known(CKnownObject::MaxAlign),
    ] {
        assert_eq!(
            ast.size_of(ty.clone()).unwrap().ty(),
            &CObjectType::scalar(T::Size)
        );
        assert_eq!(
            ast.align_of(ty).unwrap().ty(),
            &CObjectType::scalar(T::Size)
        );
    }
    let opaque = CObjectType::known(CKnownObject::File);
    assert!(matches!(ast.size_of(opaque.clone()), Err(E::Type(_))));
    assert!(matches!(ast.align_of(opaque), Err(E::Type(_))));
}
