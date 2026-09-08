//! Non-operator expression payloads retain variants, aliases and ordered children.
use super::registry_nominals::{key, registry};
use crate::ast::{
    CConstness as Q, CConversion, CExpressions, CKnownConstant, CLiteral, CNullPointer,
    CObjectType, CPointerTarget, CPointerTest, CScalarType as T, CSignedLiteral, CValueKind as K,
};
#[test]
fn conditional_numeric_pointer_and_layout_payloads_are_exact() {
    let (mut registry, file) = registry();
    let scalar = CObjectType::scalar(T::I32);
    let enumeration = registry.declare_enum(&file, key("Status")).unwrap();
    let enumerator = registry
        .register_enumerator(&enumeration, key("First"), 7)
        .unwrap();
    registry
        .define_enum(&enumeration, vec![enumerator.clone()])
        .unwrap();
    let object = registry
        .register_object(&file, key("global"), scalar.clone())
        .unwrap();
    let alias = registry
        .register_typedef(
            &file,
            key("ConstContext"),
            CObjectType::pointer(CPointerTarget::Void(Q::Const)),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let condition = ast.literal(CLiteral::Bool(true)).unwrap();
    let left = ast
        .literal(CLiteral::Signed(CSignedLiteral::I32(3)))
        .unwrap();
    let right = ast
        .literal(CLiteral::Signed(CSignedLiteral::I32(8)))
        .unwrap();
    assert_eq!(
        ast.conditional(condition.clone(), left.clone(), right.clone())
            .unwrap()
            .kind(),
        &K::Conditional {
            condition: Box::new(condition),
            then_value: Box::new(left.clone()),
            else_value: Box::new(right)
        }
    );
    assert_eq!(
        ast.enumerator(enumerator.clone()).unwrap().kind(),
        &K::Enumerator(enumerator)
    );
    for destination in T::ALL {
        assert_eq!(
            ast.numeric_conversion(destination, left.clone())
                .unwrap()
                .kind(),
            &K::Convert {
                conversion: CConversion::Numeric(destination),
                operand: Box::new(left.clone())
            }
        );
    }
    let operand = ast.address_of(ast.global(object).unwrap()).unwrap();
    let destination = CObjectType::typedef(alias);
    assert_eq!(
        ast.object_to_void(destination.clone(), operand.clone())
            .unwrap()
            .kind(),
        &K::Convert {
            conversion: CConversion::ObjectToVoid(destination),
            operand: Box::new(operand)
        }
    );
    for constant in [
        CKnownConstant::StandardInput,
        CKnownConstant::StandardOutput,
        CKnownConstant::StandardError,
    ] {
        assert_eq!(
            ast.known_constant(constant).kind(),
            &K::KnownConstant(constant)
        );
    }
    let pointer = ast
        .literal(CLiteral::NullPointer(
            CNullPointer::new(CObjectType::pointer(CPointerTarget::Object(Box::new(
                scalar.clone(),
            ))))
            .unwrap(),
        ))
        .unwrap();
    for test in [
        CPointerTest::IsNull(Box::new(pointer.clone())),
        CPointerTest::IsNonNull(Box::new(pointer)),
    ] {
        assert_eq!(
            ast.pointer_test(test.clone()).unwrap().kind(),
            &K::PointerTest(test)
        );
    }
    assert_eq!(
        ast.size_of(scalar.clone()).unwrap().kind(),
        &K::SizeOf(scalar.clone())
    );
    assert_eq!(
        ast.align_of(scalar.clone()).unwrap().kind(),
        &K::AlignOf(scalar)
    );
}
#[test]
fn indirect_nonvoid_calls_preserve_alias_signature_pointer_and_ordered_arguments() {
    use crate::ast::{
        CCallableKind, CExpressionError, CFunctionType, CParameterType, CReturnType, CReturnValue,
    };
    let (mut registry, file) = registry();
    let alias = registry
        .register_typedef(&file, key("Number"), CObjectType::scalar(T::I32))
        .unwrap();
    let ty = CObjectType::typedef(alias);
    let function = registry
        .register_function(
            &file,
            key("combine"),
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
                vec![CParameterType::new(ty).unwrap(); 2],
            ),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let pointer = ast.function_address(function.clone()).unwrap();
    let callable = ast.indirect(pointer.clone(), function.clone()).unwrap();
    assert_eq!(
        callable.kind(),
        &CCallableKind::Indirect {
            pointer: Box::new(pointer),
            contract_function: Box::new(function)
        }
    );
    let arguments = [2, 9]
        .map(|value| {
            ast.literal(CLiteral::Signed(CSignedLiteral::I32(value)))
                .unwrap()
        })
        .to_vec();
    let result = ast.call_value(callable.clone(), arguments.clone()).unwrap();
    let K::Call(call) = result.kind() else {
        panic!("indirect value call")
    };
    assert_eq!(call.callable(), &callable);
    assert_eq!(call.arguments(), arguments);
    assert_eq!(result.ty(), &CObjectType::scalar(T::I32));
    assert_eq!(
        ast.call_effect(callable.clone(), arguments.clone()),
        Err(CExpressionError::ExpectedEffectCall)
    );
    for position in 0..2 {
        let mut wrong = arguments.clone();
        wrong[position] = ast.literal(CLiteral::Bool(true)).unwrap();
        assert_eq!(
            ast.call_value(callable.clone(), wrong),
            Err(CExpressionError::TypeMismatch)
        );
    }
}
