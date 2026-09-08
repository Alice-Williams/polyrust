//! Place shape, array address formation and exact member qualifier propagation.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CArrayLength, CConstness, CExpressionError as E, CExpressions, CFunctionType,
    CIndexBase, CKnownObject, CLiteral, CObjectType, CObjectTypeKind, CParameterType,
    CPointerTarget, CRegistryError, CReturnType, CScalarType as T, CUnsignedLiteral,
};

fn scalar() -> CObjectType {
    CObjectType::scalar(T::I32)
}
fn pointer(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}

#[test]
fn locals_parameters_and_globals_retain_declared_owners() {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(
            &file,
            key("run"),
            CFunctionType::new(
                CReturnType::Void,
                vec![CParameterType::new(scalar()).unwrap()],
            ),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    let local = registry
        .register_local(&scope, key("local"), scalar())
        .unwrap();
    let parameter = registry
        .register_parameter(&function, 0, key("parameter"), CConstness::Const)
        .unwrap();
    let global = registry
        .register_object(&file, key("global"), scalar())
        .unwrap();
    let ast = CExpressions::new(&registry);
    let places = [
        ast.local(local).unwrap(),
        ast.parameter(parameter).unwrap(),
        ast.global(global).unwrap(),
    ];
    for place in places {
        let expected = pointer(place.ty().clone());
        assert_eq!(ast.address_of(place.clone()).unwrap().ty(), &expected);
        assert_eq!(ast.read(place).unwrap().ty(), &scalar());
    }
}

#[test]
fn const_aggregate_members_qualify_array_elements_not_the_array() {
    let (mut registry, file) = registry();
    let owner = CAggregateRef::Struct(registry.declare_struct(&file, key("Record")).unwrap());
    let CAggregateRef::Struct(record) = &owner else {
        unreachable!()
    };
    let array = CObjectType::array(scalar(), CArrayLength::new(4).unwrap()).unwrap();
    let member = registry
        .register_member(&owner, key("items"), array)
        .unwrap();
    registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let global = registry
        .register_object(
            &file,
            key("record"),
            CObjectType::structure(record.clone())
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap();
    let wrong = CAggregateRef::Struct(registry.declare_struct(&file, key("Wrong")).unwrap());
    let wrong_member = registry
        .register_member(&wrong, key("items"), scalar())
        .unwrap();
    let ast = CExpressions::new(&registry);
    let base = ast.global(global).unwrap();
    assert_eq!(
        ast.member(base.clone(), wrong_member),
        Err(E::Registry(CRegistryError::WrongOwner))
    );
    let array_place = ast.member(base, member).unwrap();
    assert_eq!(array_place.ty().constness(), CConstness::Unqualified);
    let CObjectTypeKind::Array { element, .. } = array_place.ty().kind() else {
        panic!("array member")
    };
    assert_eq!(element.constness(), CConstness::Const);
    assert_eq!(
        ast.read(array_place.clone()),
        Err(E::ArrayReadRequiresExplicitAddress)
    );
    let index = ast
        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
        .unwrap();
    let item = ast
        .index(CIndexBase::Array(Box::new(array_place)), index)
        .unwrap();
    assert_eq!(item.ty().constness(), CConstness::Const);
    assert_eq!(
        ast.address_of(item).unwrap().ty(),
        &pointer(scalar().with_constness(CConstness::Const).unwrap())
    );
}

#[test]
fn dereference_and_index_require_object_pointer_or_array_categories() {
    let (mut registry, file) = registry();
    let value = registry
        .register_object(&file, key("pointer"), pointer(scalar()))
        .unwrap();
    let stream = registry
        .register_object(
            &file,
            key("stream"),
            pointer(CObjectType::known(CKnownObject::File)),
        )
        .unwrap();
    let floating = registry
        .register_object(&file, key("floating_value"), CObjectType::scalar(T::F64))
        .unwrap();
    let ast = CExpressions::new(&registry);
    let value = ast.read(ast.global(value).unwrap()).unwrap();
    let zero = ast
        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
        .unwrap();
    assert_eq!(ast.dereference(value.clone()).unwrap().ty(), &scalar());
    assert_eq!(
        ast.index(CIndexBase::Pointer(Box::new(value.clone())), zero.clone())
            .unwrap()
            .ty(),
        &scalar()
    );
    assert_eq!(ast.dereference(zero.clone()), Err(E::ExpectedObjectPointer));
    let double = ast.read(ast.global(floating).unwrap()).unwrap();
    assert!(matches!(
        ast.index(CIndexBase::Pointer(Box::new(value)), double),
        Err(E::Operator(_))
    ));
    let stream = ast.read(ast.global(stream).unwrap()).unwrap();
    assert!(matches!(
        ast.read(ast.dereference(stream.clone()).unwrap()),
        Err(E::Type(_))
    ));
    assert!(matches!(
        ast.index(CIndexBase::Pointer(Box::new(stream)), zero),
        Err(E::Type(_))
    ));
}
