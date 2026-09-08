//! Storage references and path children survive construction exactly.
use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CArrayLength, CConstness, CExpressions, CFunctionType, CIndexBase, CLiteral,
    CObjectType, CParameterType, CPlaceKind as K, CPointerTarget, CReturnType, CScalarType,
    CSignedLiteral, CValueKind,
};
#[test]
fn every_place_preserves_its_origin_and_each_path_child() {
    let (mut registry, file) = registry();
    let scalar = CObjectType::scalar(CScalarType::I32);
    let function = registry
        .register_function(
            &file,
            key("run"),
            CFunctionType::new(
                CReturnType::Void,
                vec![CParameterType::new(scalar.clone()).unwrap()],
            ),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let parameter = registry
        .register_parameter(&function, 0, key("arg"), CConstness::Unqualified)
        .unwrap();
    let local = registry
        .register_local(&scope, key("local"), scalar.clone())
        .unwrap();
    let global = registry
        .register_object(&file, key("global"), scalar.clone())
        .unwrap();
    let record = registry.declare_struct(&file, key("Record")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let member = registry
        .register_member(&owner, key("item"), scalar.clone())
        .unwrap();
    registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let aggregate = registry
        .register_local(&scope, key("record"), CObjectType::structure(record))
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
    let aggregate = ast.local(aggregate).unwrap();
    let pointer = ast.read(ast.local(pointer).unwrap()).unwrap();
    let index = ast
        .literal(CLiteral::Signed(CSignedLiteral::Int(1)))
        .unwrap();
    let array_base = CIndexBase::Array(Box::new(ast.local(array).unwrap()));
    let pointer_base = CIndexBase::Pointer(Box::new(pointer.clone()));
    let cases = [
        (ast.local(local.clone()).unwrap(), K::Local(local)),
        (
            ast.parameter(parameter.clone()).unwrap(),
            K::Parameter(parameter),
        ),
        (ast.global(global.clone()).unwrap(), K::Global(global)),
        (
            ast.member(aggregate.clone(), member.clone()).unwrap(),
            K::Member {
                base: Box::new(aggregate),
                member,
            },
        ),
        (
            ast.dereference(pointer.clone()).unwrap(),
            K::Dereference(Box::new(pointer)),
        ),
        (
            ast.index(array_base.clone(), index.clone()).unwrap(),
            K::Index {
                base: array_base,
                index: Box::new(index.clone()),
            },
        ),
        (
            ast.index(pointer_base.clone(), index.clone()).unwrap(),
            K::Index {
                base: pointer_base,
                index: Box::new(index),
            },
        ),
    ];
    for (place, expected) in cases {
        assert_eq!(place.kind(), &expected);
        assert_eq!(
            ast.read(place.clone()).unwrap().kind(),
            &CValueKind::Read(Box::new(place.clone()))
        );
        assert_eq!(
            ast.address_of(place.clone()).unwrap().kind(),
            &CValueKind::AddressOf(Box::new(place))
        );
    }
}
