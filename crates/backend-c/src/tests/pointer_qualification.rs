//! Exact immediate-pointee transitions and owning-slot categories.
use super::registry_nominals::registry;
use crate::ast::{
    CArrayLength, CConstness as Q, CConversion, CExpressionError as E, CExpressions, CLiteral,
    CNullPointer, CObjectType, CPointerTarget, CPointerTest, CScalarType, CValue, CValueKind,
};
fn pointer(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}
fn array(ty: CObjectType, length: u64) -> CObjectType {
    CObjectType::array(ty, CArrayLength::new(length).unwrap()).unwrap()
}
fn null(ast: &CExpressions<'_>, ty: CObjectType) -> CValue {
    ast.literal(CLiteral::NullPointer(CNullPointer::new(ty).unwrap()))
        .unwrap()
}
#[test]
fn add_const_is_an_exact_transition_through_array_layers_only() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let scalar = CObjectType::scalar(CScalarType::I32);
    let qualified = scalar.clone().with_constness(Q::Const).unwrap();
    let pointer_scalar = pointer(scalar.clone());
    let pairs = [
        (pointer(scalar.clone()), pointer(qualified.clone())),
        (
            pointer(array(scalar.clone(), 2)),
            pointer(array(qualified.clone(), 2)),
        ),
        (
            pointer(array(array(scalar.clone(), 2), 3)),
            pointer(array(array(qualified.clone(), 2), 3)),
        ),
        (
            pointer(pointer_scalar.clone()),
            pointer(pointer_scalar.with_constness(Q::Const).unwrap()),
        ),
        (
            CObjectType::pointer(CPointerTarget::Void(Q::Unqualified)),
            CObjectType::pointer(CPointerTarget::Void(Q::Const)),
        ),
    ];
    for (source, destination) in pairs {
        let operand = null(&ast, source.clone());
        let result = ast.add_const(destination.clone(), operand.clone()).unwrap();
        assert_eq!(
            result.kind(),
            &CValueKind::Convert {
                conversion: CConversion::AddConst(destination.clone()),
                operand: Box::new(operand),
            }
        );
        assert_eq!(
            ast.add_const(destination.clone(), null(&ast, destination.clone())),
            Err(E::InvalidPointerConversion)
        );
        assert_eq!(
            ast.add_const(source.clone(), null(&ast, source.clone())),
            Err(E::InvalidPointerConversion)
        );
        assert_eq!(
            ast.add_const(source, null(&ast, destination)),
            Err(E::InvalidPointerConversion)
        );
    }
    for destination in [
        pointer(array(qualified.clone(), 3)),
        pointer(array(
            CObjectType::scalar(CScalarType::U32)
                .with_constness(Q::Const)
                .unwrap(),
            2,
        )),
    ] {
        assert_eq!(
            ast.add_const(destination, null(&ast, pointer(array(scalar.clone(), 2)))),
            Err(E::InvalidPointerConversion)
        );
    }
    assert_eq!(
        ast.add_const(
            pointer(pointer(qualified)),
            null(&ast, pointer(pointer(scalar)))
        ),
        Err(E::InvalidPointerConversion)
    );
}
#[test]
fn same_slot_rejects_borrowed_slots_even_when_both_types_match() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let scalar = CObjectType::scalar(CScalarType::I32);
    let qualified = scalar.clone().with_constness(Q::Const).unwrap();
    for (target, accepted) in [
        (pointer(scalar.clone()), true),
        (pointer(array(scalar, 2)), true),
        (
            CObjectType::pointer(CPointerTarget::Void(Q::Unqualified)),
            true,
        ),
        (pointer(qualified.clone()), false),
        (pointer(array(array(qualified, 2), 3)), false),
        (CObjectType::pointer(CPointerTarget::Void(Q::Const)), false),
    ] {
        let slot = null(&ast, pointer(target));
        let test = CPointerTest::SameSlot {
            left: Box::new(slot.clone()),
            right: Box::new(slot),
        };
        let result = ast.pointer_test(test.clone());
        if accepted {
            assert_eq!(result.unwrap().kind(), &CValueKind::PointerTest(test));
        } else {
            assert_eq!(result, Err(E::ExpectedOwningSlotPointer));
        }
    }
}
