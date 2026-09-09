//! Private reconstruction does not confer authentic membership or valid roles.
use super::*;
use crate::ast::{
    CContextError, CDeclarations, CFileItem, CObjectType, CScalarType,
    member_role_contracts::member,
    numeric_fixture::Fixture,
    storage_fixture::{aggregate_source, pointer_type},
};

#[test]
fn constructed_wrapper_or_substituted_role_is_not_registered() {
    let mut f = Fixture::new(&[]);
    let m = member(
        &mut f,
        "Parent",
        pointer_type(CObjectType::scalar(CScalarType::Int)),
        false,
    );
    let forged = CMemberOwnershipRef {
        member: m.clone(),
        role: CMemberOwnership::Required,
    };
    assert_eq!(
        f.registry.check_member_ownership(m.owner(), &forged),
        Err(CRegistryError::UnregisteredReference)
    );
    let mut registered = f
        .registry
        .register_member_ownership(&m, CMemberOwnership::Required)
        .unwrap();
    registered.role = CMemberOwnership::Optional;
    assert_eq!(
        f.registry.check_member_ownership(m.owner(), &registered),
        Err(CRegistryError::UnregisteredReference)
    );
}

#[test]
fn an_extra_mismatched_map_key_cannot_hide_behind_a_valid_role() {
    for corrupt in [false, true] {
        let mut f = Fixture::new(&[]);
        let ty = pointer_type(CObjectType::scalar(CScalarType::Int));
        let first = member(&mut f, "First", ty.clone(), false);
        let second = member(&mut f, "Second", ty, false);
        let value = f
            .registry
            .register_member_ownership(&second, CMemberOwnership::Optional)
            .unwrap();
        if corrupt {
            f.registry.member_ownership.insert(first.clone(), value);
        }
        let source = aggregate_source(&f, first.owner().clone(), vec![]);
        let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
        let mut items = vec![CFileItem::Declaration(
            declarations.aggregate(second.owner().clone()).unwrap(),
        )];
        items.extend(source.items().iter().cloned());
        let source = declarations.source_file(items).unwrap();
        assert_eq!(
            f.registry.check_package_structure(&[source]),
            if corrupt {
                Err(CContextError::Registry(
                    CRegistryError::UnregisteredReference,
                ))
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn invalid_private_inventory_role_is_revalidated_against_actual_member() {
    let mut f = Fixture::new(&[]);
    let m = member(
        &mut f,
        "Parent",
        CObjectType::scalar(CScalarType::Int),
        false,
    );
    f.registry.member_ownership.insert(
        m.clone(),
        CMemberOwnershipRef {
            member: m.clone(),
            role: CMemberOwnership::Required,
        },
    );
    let source = aggregate_source(&f, m.owner().clone(), vec![]);
    assert_eq!(
        f.registry.check_package_structure(&[source]),
        Err(CContextError::Registry(
            CRegistryError::InvalidMemberOwnership
        ))
    );
}

#[test]
fn mismatched_authoritative_key_cannot_authenticate_a_different_member() {
    let mut f = Fixture::new(&[]);
    let ty = pointer_type(CObjectType::scalar(CScalarType::Int));
    let first = member(&mut f, "First", ty.clone(), false);
    let second = member(&mut f, "Second", ty, false);
    let forged = CMemberOwnershipRef {
        member: second.clone(),
        role: CMemberOwnership::Optional,
    };
    f.registry.member_ownership.insert(first, forged.clone());
    assert_eq!(
        f.registry.check_member_ownership(second.owner(), &forged),
        Err(CRegistryError::UnregisteredReference)
    );
}
