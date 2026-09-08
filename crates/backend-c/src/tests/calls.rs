//! Exact arity/result categories and callable-contract substitution controls.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CExpressionError as E, CExpressions, CFunctionType, CLiteral, CObjectType,
    CParameterType, CRegistryError, CReturnType, CReturnValue, CScalarType as T, CSignedLiteral,
    CValueKind,
};

fn scalar() -> CObjectType {
    CObjectType::scalar(T::I32)
}

#[test]
fn direct_calls_check_high_arity_every_argument_and_value_effect_categories() {
    let (mut registry, file) = registry();
    let parameters = vec![CParameterType::new(scalar()).unwrap(); 128];
    let value = registry
        .register_function(
            &file,
            key("value"),
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(scalar()).unwrap()),
                parameters.clone(),
            ),
        )
        .unwrap();
    let effect = registry
        .register_function(
            &file,
            key("effect"),
            CFunctionType::new(CReturnType::Void, parameters),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let argument = ast
        .literal(CLiteral::Signed(CSignedLiteral::Int(1)))
        .unwrap();
    let arguments = vec![argument; 128];
    let callable = ast.direct(value).unwrap();
    let call = ast.call_value(callable.clone(), arguments.clone()).unwrap();
    let CValueKind::Call(actual) = call.kind() else {
        panic!("call value")
    };
    assert_eq!(actual.arguments(), arguments);
    assert_eq!(call.ty(), &scalar());
    assert_eq!(
        ast.call_value(callable.clone(), arguments[..127].to_vec()),
        Err(E::ArityMismatch {
            expected: 128,
            actual: 127
        })
    );
    assert_eq!(
        ast.call_effect(callable.clone(), arguments.clone()),
        Err(E::ExpectedEffectCall)
    );
    for index in 0..128 {
        let mut bad = arguments.clone();
        bad[index] = ast.literal(CLiteral::Bool(true)).unwrap();
        assert_eq!(ast.call_value(callable.clone(), bad), Err(E::TypeMismatch));
    }
    let callable = ast.direct(effect).unwrap();
    assert!(ast.call_effect(callable.clone(), arguments.clone()).is_ok());
    assert_eq!(
        ast.call_value(callable, arguments),
        Err(E::ExpectedValueCall)
    );
}

#[test]
fn indirect_function_addresses_and_bound_members_reject_same_prototype_wrong_contract() {
    let (mut registry, file) = registry();
    let signature = CFunctionType::new(CReturnType::Void, vec![]);
    let first = registry
        .register_function(&file, key("first"), signature.clone())
        .unwrap();
    let second = registry
        .register_function(&file, key("second"), signature)
        .unwrap();
    let table = registry.declare_struct(&file, key("Table")).unwrap();
    let owner = CAggregateRef::Struct(table.clone());
    let member = registry
        .register_callable_member(&owner, key("invoke"), &first)
        .unwrap();
    registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let object = registry
        .register_object(&file, key("table"), CObjectType::structure(table))
        .unwrap();
    let ast = CExpressions::new(&registry);
    let values = [
        ast.function_address(first.clone()).unwrap(),
        ast.read(ast.member(ast.global(object).unwrap(), member).unwrap())
            .unwrap(),
    ];
    for value in values {
        let callable = ast.indirect(value.clone(), first.clone()).unwrap();
        assert_eq!(callable.contract(), first.contract());
        assert!(ast.call_effect(callable, vec![]).is_ok());
        assert_eq!(
            ast.indirect(value, second.clone()),
            Err(E::Registry(CRegistryError::CallableContractMismatch))
        );
    }
    let scalar = ast.literal(CLiteral::Bool(false)).unwrap();
    assert_eq!(ast.indirect(scalar, first), Err(E::ExpectedFunctionPointer));
}

#[test]
fn scalar_only_foreign_callables_and_arguments_are_still_rejected() {
    let (mut a, file) = registry();
    let (b, _) = registry();
    let function = a
        .register_function(
            &file,
            key("run"),
            CFunctionType::new(
                CReturnType::Void,
                vec![CParameterType::new(scalar()).unwrap()],
            ),
        )
        .unwrap();
    let a = CExpressions::new(&a);
    let b = CExpressions::new(&b);
    let callable = a.direct(function).unwrap();
    let foreign = b.literal(CLiteral::Signed(CSignedLiteral::I32(1))).unwrap();
    let error = E::Registry(CRegistryError::CrossRegistry);
    assert_eq!(
        a.call_effect(callable.clone(), vec![foreign.clone()]),
        Err(error)
    );
    assert_eq!(b.call_effect(callable, vec![foreign]), Err(error));
}
