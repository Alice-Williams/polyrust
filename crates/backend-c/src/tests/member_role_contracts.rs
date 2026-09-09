//! Member obligations retain actual declarations; registration proves no child.
use super::{
    contextual_reconstruction::key,
    numeric_fixture::Fixture,
    storage_fixture::{aggregate_source, pointer_type},
    *,
};

pub(super) fn member(f: &mut Fixture, name: &str, ty: CObjectType, union: bool) -> CMemberRef {
    let owner = if union {
        CAggregateRef::Union(f.registry.declare_union(&f.file, key(name)).unwrap())
    } else {
        CAggregateRef::Struct(f.registry.declare_struct(&f.file, key(name)).unwrap())
    };
    let member = f.registry.register_member(&owner, key("slot"), ty).unwrap();
    f.registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    member
}

#[test]
fn roles_preserve_exact_member_and_role_specific_inventory() {
    for union in [false, true] {
        for (role, kind) in [
            (CMemberOwnership::Required, CRegistrationKind::RequiredChild),
            (CMemberOwnership::Optional, CRegistrationKind::OptionalChild),
            (
                CMemberOwnership::BorrowedMetadata,
                CRegistrationKind::BorrowedMetadata,
            ),
        ] {
            let mut f = Fixture::new(&[]);
            let mut ty = CObjectType::scalar(CScalarType::Int);
            if role == CMemberOwnership::BorrowedMetadata {
                ty = ty.with_constness(CConstness::Const).unwrap();
            }
            let m = member(&mut f, "Parent", pointer_type(ty), union);
            let value = f.registry.register_member_ownership(&m, role).unwrap();
            assert_eq!(value.member(), &m);
            assert_eq!(value.role(), role);
            assert_eq!(
                f.registry.member_ownerships().collect::<Vec<_>>(),
                vec![&value]
            );
            let inventory = f.registry.inventory();
            let summary = inventory.iter().find(|v| v.kind == kind).unwrap();
            assert_eq!(summary.key, m.key().clone());
            assert_eq!(
                summary.owner,
                CRegistrationOwner::Declaration {
                    kind: if union {
                        CRegistrationKind::Union
                    } else {
                        CRegistrationKind::Struct
                    },
                    file: f.file.key().clone(),
                    key: m.owner().key().clone(),
                }
            );
            f.registry
                .freeze()
                .registrations()
                .check_member_ownership(m.owner(), &value)
                .unwrap();
        }
    }
}

#[test]
fn conflicting_duplicate_foreign_and_wrong_owner_roles_reject() {
    let mut f = Fixture::new(&[]);
    let mut foreign = Fixture::new(&[]);
    let ty = pointer_type(CObjectType::scalar(CScalarType::Int));
    let m = member(&mut f, "Parent", ty.clone(), false);
    let other = member(&mut f, "Other", ty.clone(), true);
    let foreign_m = member(&mut foreign, "Parent", ty, false);
    let value = f
        .registry
        .register_member_ownership(&m, CMemberOwnership::Required)
        .unwrap();
    let before = f.registry.inventory();
    for role in [CMemberOwnership::Required, CMemberOwnership::Optional] {
        assert_eq!(
            f.registry.register_member_ownership(&m, role),
            Err(CRegistryError::DuplicateRegistration)
        );
    }
    assert_eq!(
        f.registry
            .register_member_ownership(&foreign_m, CMemberOwnership::Required),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        f.registry.check_member_ownership(other.owner(), &value),
        Err(CRegistryError::WrongOwner)
    );
    let foreign_value = foreign
        .registry
        .register_member_ownership(&foreign_m, CMemberOwnership::Required)
        .unwrap();
    assert_eq!(
        f.registry.check_member_ownership(m.owner(), &foreign_value),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(f.registry.inventory(), before);
}

#[test]
fn actual_member_occurrence_is_required_and_role_alone_cannot_pass_safety() {
    for role in [
        None,
        Some(CMemberOwnership::Required),
        Some(CMemberOwnership::Optional),
        Some(CMemberOwnership::BorrowedMetadata),
    ] {
        let mut f = Fixture::new(&[]);
        let mut target = CObjectType::scalar(CScalarType::Int);
        if role == Some(CMemberOwnership::BorrowedMetadata) {
            target = target.with_constness(CConstness::Const).unwrap();
        }
        let m = member(&mut f, "Parent", pointer_type(target), false);
        if let Some(role) = role {
            f.registry.register_member_ownership(&m, role).unwrap();
        }
        assert_eq!(
            f.registry.check_package_structure(&[f.source(vec![])]),
            Err(CContextError::MissingRegistrationOccurrence)
        );
        let source = aggregate_source(&f, m.owner().clone(), vec![]);
        f.registry
            .check_package_structure(std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if role.is_some() {
                Err(CSafetyError::UnprovedOwnership)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn inventory_is_independent_of_registration_order_and_registry_brand() {
    let inventories: Vec<_> = [false, true]
        .into_iter()
        .map(|reverse| {
            let mut f = Fixture::new(&[]);
            for name in if reverse { ["B", "A"] } else { ["A", "B"] } {
                let m = member(
                    &mut f,
                    name,
                    pointer_type(CObjectType::scalar(CScalarType::Int)),
                    false,
                );
                f.registry
                    .register_member_ownership(&m, CMemberOwnership::Optional)
                    .unwrap();
            }
            f.registry.inventory()
        })
        .collect();
    assert_eq!(inventories[0], inventories[1]);
}

#[test]
fn forward_nominal_child_role_does_not_claim_pointee_layout() {
    let mut f = Fixture::new(&[]);
    let child = f.registry.declare_struct(&f.file, key("Forward")).unwrap();
    let m = member(
        &mut f,
        "Parent",
        pointer_type(CObjectType::structure(child.clone())),
        false,
    );
    f.registry
        .register_member_ownership(&m, CMemberOwnership::Required)
        .unwrap();
    assert_eq!(
        f.registry
            .members(&CAggregateRef::Struct(child.clone()))
            .unwrap(),
        None
    );
    let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
    let source = aggregate_source(&f, m.owner().clone(), vec![]);
    let mut items = vec![CFileItem::Declaration(
        declarations
            .forward_tag(CAggregateRef::Struct(child))
            .unwrap(),
    )];
    items.extend(source.items().iter().cloned());
    let source = declarations.source_file(items).unwrap();
    f.registry
        .check_package_structure(std::slice::from_ref(&source))
        .unwrap();
    assert_eq!(
        f.registry.check_storage_paths(&[source]),
        Err(CSafetyError::UnprovedOwnership)
    );
}
