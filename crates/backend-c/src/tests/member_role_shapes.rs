//! Closed role categories, nested fixed-array terminals and authentic aliases.
use super::{
    contextual_reconstruction::key, member_role_contracts::member, numeric_fixture::Fixture,
    storage_fixture::pointer_type, *,
};

fn array(ty: CObjectType) -> CObjectType {
    CObjectType::array(ty, CArrayLength::new(1_000_000).unwrap()).unwrap()
}
fn callback() -> CObjectType {
    CObjectType::pointer(CPointerTarget::Function(Box::new(CFunctionType::new(
        CReturnType::Void,
        vec![],
    ))))
}
fn check_shape(ty: CObjectType, role: CMemberOwnership, valid: bool) {
    let mut f = Fixture::new(&[]);
    let m = member(&mut f, "Parent", ty, false);
    let before = f.registry.inventory();
    let result = f.registry.register_member_ownership(&m, role);
    if valid {
        let value = result.unwrap();
        f.registry
            .check_member_ownership(m.owner(), &value)
            .unwrap();
    } else {
        assert_eq!(result, Err(CRegistryError::InvalidMemberOwnership));
        assert_eq!(f.registry.inventory(), before);
        assert_eq!(f.registry.member_ownerships().count(), 0);
    }
}

#[test]
fn required_and_optional_accept_only_mutable_object_pointer_terminals() {
    let int = CObjectType::scalar(CScalarType::Int);
    let borrowed = int.clone().with_constness(CConstness::Const).unwrap();
    for role in [CMemberOwnership::Required, CMemberOwnership::Optional] {
        for ty in [
            pointer_type(int.clone()),
            array(array(pointer_type(int.clone()))),
            pointer_type(array(int.clone())),
        ] {
            check_shape(ty, role, true);
        }
        for ty in [
            int.clone(),
            array(int.clone()),
            pointer_type(borrowed.clone()),
            pointer_type(array(borrowed.clone())),
            pointer_type(int.clone())
                .with_constness(CConstness::Const)
                .unwrap(),
            array(
                pointer_type(int.clone())
                    .with_constness(CConstness::Const)
                    .unwrap(),
            ),
            callback(),
            CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified)),
            pointer_type(CObjectType::known(CKnownObject::File)),
        ] {
            check_shape(ty, role, false);
        }
    }
}

#[test]
fn metadata_is_static_const_object_or_function_category_not_an_owner() {
    let int = CObjectType::scalar(CScalarType::Int);
    let borrowed = int.clone().with_constness(CConstness::Const).unwrap();
    for ty in [
        pointer_type(borrowed.clone()),
        pointer_type(array(borrowed.clone())),
        array(pointer_type(borrowed.clone())),
        pointer_type(borrowed)
            .with_constness(CConstness::Const)
            .unwrap(),
        callback(),
        array(callback()),
    ] {
        check_shape(ty, CMemberOwnership::BorrowedMetadata, true);
    }
    for ty in [
        int.clone(),
        pointer_type(int),
        CObjectType::pointer(CPointerTarget::Void(CConstness::Const)),
        pointer_type(
            CObjectType::known(CKnownObject::File)
                .with_constness(CConstness::Const)
                .unwrap(),
        ),
    ] {
        check_shape(ty, CMemberOwnership::BorrowedMetadata, false);
    }
}

#[test]
fn source_aliases_remain_authenticated_through_array_and_pointer_layers() {
    for role in [
        CMemberOwnership::Required,
        CMemberOwnership::Optional,
        CMemberOwnership::BorrowedMetadata,
    ] {
        for valid in [false, true] {
            let mut f = Fixture::new(&[]);
            let mut target = CObjectType::scalar(CScalarType::Int);
            if (role == CMemberOwnership::BorrowedMetadata) == valid {
                target = target.with_constness(CConstness::Const).unwrap();
            }
            let target = f
                .registry
                .register_typedef(&f.file, key("Target"), target)
                .unwrap();
            let alias = f
                .registry
                .register_typedef(
                    &f.file,
                    key("Slots"),
                    array(pointer_type(CObjectType::typedef(target))),
                )
                .unwrap();
            let declared = CObjectType::typedef(alias);
            let m = member(&mut f, "Parent", declared.clone(), false);
            let result = f.registry.register_member_ownership(&m, role);
            if valid {
                assert_eq!(result.unwrap().member().ty(), &declared);
            } else {
                assert_eq!(result, Err(CRegistryError::InvalidMemberOwnership));
            }
        }
    }
}

#[test]
fn inline_aggregates_cannot_be_relabelled_as_pointer_slots() {
    let mut f = Fixture::new(&[]);
    let nested = member(
        &mut f,
        "Nested",
        pointer_type(CObjectType::scalar(CScalarType::Int)),
        false,
    );
    f.registry
        .register_member_ownership(&nested, CMemberOwnership::Optional)
        .unwrap();
    let CAggregateRef::Struct(tag) = nested.owner() else {
        unreachable!()
    };
    let ty = CObjectType::structure(tag.clone());
    let inline = member(&mut f, "Parent", array(ty), false);
    assert_eq!(
        f.registry
            .register_member_ownership(&inline, CMemberOwnership::Optional),
        Err(CRegistryError::InvalidMemberOwnership)
    );
    assert_eq!(f.registry.member_ownerships().count(), 1);
}
