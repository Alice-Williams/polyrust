//! Explicit pointer conversions cannot smuggle qualifier loss or truthiness.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CArrayLength, CBinaryOperator, CConstness as Q, CExpressionError as E, CExpressions,
    CFunctionType, CLiteral, CNullPointer, CObjectType, CPointerTarget, CPointerTest, CReturnType,
    CScalarType as T,
};

fn scalar() -> CObjectType {
    CObjectType::scalar(T::I32)
}
fn pointer(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}
fn void(qualifier: Q) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Void(qualifier))
}
fn null(ast: &CExpressions<'_>, ty: CObjectType) -> crate::ast::CValue {
    ast.literal(CLiteral::NullPointer(CNullPointer::new(ty).unwrap()))
        .unwrap()
}

#[test]
fn pointer_tests_are_separate_int_results_not_arithmetic_or_truthy_conditions() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let function = CObjectType::pointer(CPointerTarget::Function(Box::new(CFunctionType::new(
        CReturnType::Void,
        vec![],
    ))));
    for ty in [pointer(scalar()), void(Q::Unqualified), function] {
        let value = null(&ast, ty);
        for test in [
            CPointerTest::IsNull(Box::new(value.clone())),
            CPointerTest::IsNonNull(Box::new(value.clone())),
        ] {
            let result = ast.pointer_test(test).unwrap();
            assert_eq!(result.ty(), &CObjectType::scalar(T::Int));
            assert_eq!(
                ast.numeric_conversion(T::Bool, result).unwrap().ty(),
                &CObjectType::scalar(T::Bool)
            );
        }
        assert_eq!(
            ast.binary(CBinaryOperator::Equal, value.clone(), value.clone()),
            Err(E::ExpectedArithmetic)
        );
        assert_eq!(
            ast.conditional(value.clone(), value.clone(), value.clone()),
            Err(E::ExpectedBool)
        );
        assert_eq!(
            ast.numeric_conversion(T::Bool, value),
            Err(E::ExpectedArithmetic)
        );
    }
    let truth = ast.literal(CLiteral::Bool(true)).unwrap();
    assert!(matches!(
        ast.pointer_test(CPointerTest::IsNull(Box::new(truth))),
        Err(E::Type(_))
    ));
}

#[test]
fn same_slot_requires_matching_mutable_slot_address_types() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let slot = null(&ast, pointer(pointer(scalar())));
    assert!(
        ast.pointer_test(CPointerTest::SameSlot {
            left: Box::new(slot.clone()),
            right: Box::new(slot.clone())
        })
        .is_ok()
    );
    for other in [
        pointer(scalar()),
        pointer(pointer(CObjectType::scalar(T::U32))),
        pointer(pointer(scalar()).with_constness(Q::Const).unwrap()),
    ] {
        assert_eq!(
            ast.pointer_test(CPointerTest::SameSlot {
                left: Box::new(slot.clone()),
                right: Box::new(null(&ast, other))
            }),
            Err(E::ExpectedOwningSlotPointer)
        );
    }
    let live = null(&ast, pointer(scalar()));
    assert_eq!(
        ast.pointer_test(CPointerTest::SameSlot {
            left: Box::new(live.clone()),
            right: Box::new(live)
        }),
        Err(E::ExpectedOwningSlotPointer)
    );
}

#[test]
fn adding_const_changes_only_the_immediate_pointee() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let qualified = scalar().with_constness(Q::Const).unwrap();
    assert!(
        ast.add_const(pointer(qualified.clone()), null(&ast, pointer(scalar())))
            .is_ok()
    );
    assert_eq!(
        ast.add_const(pointer(scalar()), null(&ast, pointer(qualified.clone()))),
        Err(E::InvalidPointerConversion)
    );
    assert_eq!(
        ast.add_const(
            pointer(pointer(qualified)),
            null(&ast, pointer(pointer(scalar())))
        ),
        Err(E::InvalidPointerConversion)
    );
    assert!(
        ast.add_const(
            pointer(pointer(scalar()).with_constness(Q::Const).unwrap()),
            null(&ast, pointer(pointer(scalar())))
        )
        .is_ok()
    );
    assert!(
        ast.add_const(void(Q::Const), null(&ast, void(Q::Unqualified)))
            .is_ok()
    );
    assert_eq!(
        ast.add_const(void(Q::Unqualified), null(&ast, void(Q::Const))),
        Err(E::InvalidPointerConversion)
    );
}

#[test]
fn object_erasure_preserves_const_including_array_elements() {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(
            &file,
            key("run"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let qualified = scalar().with_constness(Q::Const).unwrap();
    for ty in [
        qualified.clone(),
        CObjectType::array(qualified, CArrayLength::new(2).unwrap()).unwrap(),
    ] {
        let value = null(&ast, pointer(ty));
        assert_eq!(
            ast.object_to_void(void(Q::Unqualified), value.clone()),
            Err(E::InvalidPointerConversion)
        );
        assert!(ast.object_to_void(void(Q::Const), value).is_ok());
    }
    assert!(
        ast.object_to_void(void(Q::Unqualified), null(&ast, pointer(scalar())))
            .is_ok()
    );
    assert_eq!(
        ast.object_to_void(void(Q::Const), ast.function_address(function).unwrap()),
        Err(E::InvalidPointerConversion)
    );
    assert_eq!(
        ast.object_to_void(pointer(scalar()), null(&ast, void(Q::Unqualified))),
        Err(E::InvalidPointerConversion)
    );
}
