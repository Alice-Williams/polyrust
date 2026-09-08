//! Every initializer category has positive and invalid-shape controls.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CArrayLength, CConstness, CExpressions, CInitializerError as E, CLiteral,
    CObjectType, CRegistryError, CScalarType as T, CSignedLiteral,
};

fn scalar() -> CObjectType {
    CObjectType::scalar(T::I32)
}

#[test]
fn expression_zero_and_array_initializers_require_exact_shape_and_brand() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    let one = ast
        .expression_initializer(
            ast.literal(CLiteral::Signed(CSignedLiteral::I32(1)))
                .unwrap(),
        )
        .unwrap();
    let zero = ast.zero_initializer(scalar()).unwrap();
    assert_eq!(one.ty(), &scalar());
    assert_eq!(zero.ty(), &scalar());
    let array = CObjectType::array(
        scalar().with_constness(CConstness::Const).unwrap(),
        CArrayLength::new(2).unwrap(),
    )
    .unwrap();
    assert!(
        ast.array_initializer(array.clone(), vec![one.clone(), zero.clone()])
            .is_ok()
    );
    assert_eq!(
        ast.array_initializer(array.clone(), vec![one.clone()]),
        Err(E::ElementCount {
            expected: 2,
            actual: 1
        })
    );
    assert_eq!(
        ast.array_initializer(array.clone(), vec![one.clone(), zero.clone(), one.clone()]),
        Err(E::ElementCount {
            expected: 2,
            actual: 3
        })
    );
    assert_eq!(
        ast.array_initializer(scalar(), vec![one.clone()]),
        Err(E::ExpectedArray)
    );
    let truth = ast.zero_initializer(CObjectType::scalar(T::Bool)).unwrap();
    assert_eq!(
        ast.array_initializer(array.clone(), vec![one.clone(), truth]),
        Err(E::TypeMismatch)
    );
    let (foreign, _) = super::registry_nominals::registry();
    let foreign = CExpressions::new(&foreign)
        .zero_initializer(scalar())
        .unwrap();
    assert_eq!(
        ast.array_initializer(array, vec![one, foreign]),
        Err(E::Registry(CRegistryError::CrossRegistry))
    );
}

#[test]
fn struct_initializers_reject_missing_duplicate_wrong_owner_and_reordered_members() {
    let (mut registry, file) = registry();
    let record = registry.declare_struct(&file, key("Record")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let first = registry
        .register_member(&owner, key("first"), scalar())
        .unwrap();
    let second = registry
        .register_member(&owner, key("second"), scalar())
        .unwrap();
    registry
        .define_aggregate(&owner, vec![first.clone(), second.clone()])
        .unwrap();
    let other = registry.declare_struct(&file, key("Other")).unwrap();
    let wrong = registry
        .register_member(
            &CAggregateRef::Struct(other.clone()),
            key("first"),
            scalar(),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let value = ast.zero_initializer(scalar()).unwrap();
    assert!(
        ast.struct_initializer(
            record.clone(),
            vec![
                (first.clone(), value.clone()),
                (second.clone(), value.clone())
            ]
        )
        .is_ok()
    );
    for fields in [
        vec![],
        vec![(first.clone(), value.clone())],
        vec![
            (first.clone(), value.clone()),
            (first.clone(), value.clone()),
        ],
        vec![(second, value.clone()), (first.clone(), value.clone())],
    ] {
        assert_eq!(
            ast.struct_initializer(record.clone(), fields),
            Err(E::MemberInventoryMismatch)
        );
    }
    assert_eq!(
        ast.struct_initializer(record.clone(), vec![(wrong, value.clone())]),
        Err(E::Registry(CRegistryError::WrongOwner))
    );
    assert_eq!(
        ast.struct_initializer(other, vec![]),
        Err(E::IncompleteAggregate)
    );
    let wrong_type = ast.zero_initializer(CObjectType::scalar(T::Bool)).unwrap();
    assert_eq!(
        ast.struct_initializer(record, vec![(first, wrong_type)]),
        Err(E::TypeMismatch)
    );
}

#[test]
fn union_initializers_select_one_exact_registered_member_and_type() {
    let (mut registry, file) = registry();
    let union = registry.declare_union(&file, key("Payload")).unwrap();
    let owner = CAggregateRef::Union(union.clone());
    let number = registry
        .register_member(&owner, key("number"), scalar())
        .unwrap();
    let flag = registry
        .register_member(&owner, key("flag"), CObjectType::scalar(T::Bool))
        .unwrap();
    registry
        .define_aggregate(&owner, vec![number.clone(), flag.clone()])
        .unwrap();
    let other = registry.declare_union(&file, key("Other")).unwrap();
    let foreign = registry
        .register_member(&CAggregateRef::Union(other), key("number"), scalar())
        .unwrap();
    let ast = CExpressions::new(&registry);
    let value = ast.zero_initializer(scalar()).unwrap();
    assert!(
        ast.union_initializer(union.clone(), number, value.clone())
            .is_ok()
    );
    assert_eq!(
        ast.union_initializer(union.clone(), flag, value.clone()),
        Err(E::TypeMismatch)
    );
    assert_eq!(
        ast.union_initializer(union, foreign, value),
        Err(E::Registry(CRegistryError::WrongOwner))
    );
}
