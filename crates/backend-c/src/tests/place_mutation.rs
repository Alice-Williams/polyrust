//! Each admitted place has exact assignment payload and const/type negatives.
use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CArrayLength, CConstness as Q, CExpressions, CFunctionType, CIndexBase,
    CLiteral, CObjectType, CParameterType, CPointerTarget, CReturnType, CScalarType as T,
    CSignedLiteral, CStatementError as E, CStatementKind, CStatements,
};
#[test]
fn assignment_matrix_covers_all_place_forms_and_qualification() {
    for qualifier in [Q::Unqualified, Q::Const] {
        let (mut registry, file) = registry();
        let scalar = CObjectType::scalar(T::I32)
            .with_constness(qualifier)
            .unwrap();
        let function = registry
            .register_function(
                &file,
                key("run"),
                CFunctionType::new(
                    CReturnType::Void,
                    vec![CParameterType::new(CObjectType::scalar(T::I32)).unwrap()],
                ),
            )
            .unwrap();
        let parameter = registry
            .register_parameter(&function, 0, key("arg"), qualifier)
            .unwrap();
        let scope = registry
            .register_scope(&function, None, key("root"))
            .unwrap();
        let local = registry
            .register_local(&scope, key("local"), scalar.clone())
            .unwrap();
        let global = registry
            .register_object(&file, key("global"), scalar.clone())
            .unwrap();
        let owner = registry.declare_struct(&file, key("Record")).unwrap();
        let aggregate = CAggregateRef::Struct(owner.clone());
        let member = registry
            .register_member(&aggregate, key("value"), scalar.clone())
            .unwrap();
        registry
            .define_aggregate(&aggregate, vec![member.clone()])
            .unwrap();
        let record = registry
            .register_local(&scope, key("record"), CObjectType::structure(owner))
            .unwrap();
        let array = registry
            .register_local(
                &scope,
                key("array"),
                CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap(),
            )
            .unwrap();
        let pointer = registry
            .register_local(
                &scope,
                key("pointer"),
                CObjectType::pointer(CPointerTarget::Object(Box::new(scalar))),
            )
            .unwrap();
        let ast = CExpressions::new(&registry);
        let statements = CStatements::new(&registry, function).unwrap();
        let index = ast
            .literal(CLiteral::Signed(CSignedLiteral::Int(0)))
            .unwrap();
        let pointer = ast.read(ast.local(pointer).unwrap()).unwrap();
        let places = [
            ast.local(local).unwrap(),
            ast.global(global).unwrap(),
            ast.parameter(parameter).unwrap(),
            ast.member(ast.local(record).unwrap(), member).unwrap(),
            ast.dereference(pointer.clone()).unwrap(),
            ast.index(
                CIndexBase::Array(Box::new(ast.local(array).unwrap())),
                index.clone(),
            )
            .unwrap(),
            ast.index(CIndexBase::Pointer(Box::new(pointer)), index)
                .unwrap(),
        ];
        let value = ast
            .literal(CLiteral::Signed(CSignedLiteral::I32(5)))
            .unwrap();
        for place in places {
            let result = statements.assign(place.clone(), value.clone());
            if qualifier == Q::Const {
                assert_eq!(result, Err(E::NotModifiable));
            } else {
                assert_eq!(
                    result.unwrap().kind(),
                    &CStatementKind::Assign {
                        place: place.clone(),
                        value: value.clone(),
                    }
                );
                assert_eq!(
                    statements.assign(place, ast.literal(CLiteral::Bool(true)).unwrap()),
                    Err(E::TypeMismatch)
                );
            }
        }
    }
}
