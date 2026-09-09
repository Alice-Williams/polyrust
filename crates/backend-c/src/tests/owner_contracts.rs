//! Owner roles preserve declarations and cannot claim lifecycle success.
use super::{
    allocation_fixture::null,
    contextual_reconstruction::key,
    numeric_fixture::Fixture,
    storage_fixture::{local, pointer_type},
    *,
};

#[test]
fn local_owner_roles_retain_exact_origin_type_and_inventory() {
    let mut f = Fixture::new(&[]);
    let slot = local(
        &mut f,
        pointer_type(CObjectType::scalar(CScalarType::Int)),
        "owner",
    );
    let owner = f.registry.register_local_owner(&slot).unwrap();
    assert_eq!(owner.local(), &slot);
    assert_eq!(owner.local().scope().function(), &f.function);
    assert_eq!(f.registry.owner_slots().collect::<Vec<_>>(), vec![&owner]);
    f.registry.check_owner_slot(&f.function, &owner).unwrap();
    let summary = f
        .registry
        .inventory()
        .into_iter()
        .find(|entry| entry.kind == CRegistrationKind::OwnerSlot)
        .unwrap();
    assert_eq!(summary.key, slot.key().clone());
    assert_eq!(
        summary.owner,
        CRegistrationOwner::Scope {
            file: f.file.key().clone(),
            function: f.function.key().clone(),
            scope: f.scope.key().clone(),
        }
    );
    let frozen = f.registry.freeze();
    frozen
        .registrations()
        .check_owner_slot(&f.function, &owner)
        .unwrap();
}

#[test]
fn invalid_categories_cannot_be_relabelled_as_local_owners() {
    let int = CObjectType::scalar(CScalarType::Int);
    let borrowed = int.clone().with_constness(CConstness::Const).unwrap();
    let array = CObjectType::array(borrowed.clone(), CArrayLength::new(2).unwrap()).unwrap();
    let invalid = vec![
        int.clone(),
        pointer_type(int.clone())
            .with_constness(CConstness::Const)
            .unwrap(),
        pointer_type(borrowed),
        pointer_type(array),
        CObjectType::array(pointer_type(int), CArrayLength::new(2).unwrap()).unwrap(),
        CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified)),
        CObjectType::pointer(CPointerTarget::Function(Box::new(CFunctionType::new(
            CReturnType::Void,
            vec![],
        )))),
        pointer_type(CObjectType::known(CKnownObject::File)),
    ];
    for ty in invalid {
        let mut f = Fixture::new(&[]);
        let slot = local(&mut f, ty.clone(), "invalid");
        assert_eq!(
            f.registry.register_local_owner(&slot),
            Err(CRegistryError::InvalidOwnerSlot),
            "{ty:?}"
        );
        assert_eq!(f.registry.owner_slots().count(), 0);
    }
}

#[test]
fn owner_aliases_are_authenticated_without_losing_the_declared_type() {
    for borrowed in [false, true] {
        let mut f = Fixture::new(&[]);
        let mut payload = CObjectType::scalar(CScalarType::Int);
        if borrowed {
            payload = payload.with_constness(CConstness::Const).unwrap();
        }
        let alias = f
            .registry
            .register_typedef(&f.file, key("Handle"), pointer_type(payload))
            .unwrap();
        let declared = CObjectType::typedef(alias);
        let slot = local(&mut f, declared.clone(), "owner");
        let result = f.registry.register_local_owner(&slot);
        if borrowed {
            assert_eq!(result, Err(CRegistryError::InvalidOwnerSlot));
        } else {
            assert_eq!(result.unwrap().local().ty(), &declared);
        }
    }
}

#[test]
fn ownership_requires_a_real_occurrence_and_proves_actual_empty_initialization() {
    for owned in [false, true] {
        let mut f = Fixture::new(&[]);
        let slot = local(
            &mut f,
            pointer_type(CObjectType::scalar(CScalarType::Int)),
            "owner",
        );
        if owned {
            f.registry.register_local_owner(&slot).unwrap();
        }
        let missing = f.source(vec![]);
        assert_eq!(
            f.registry.check_package_structure(&[missing]),
            Err(CContextError::MissingRegistrationOccurrence)
        );
        let source = f.source(vec![f.declare(&slot, null(&f, &slot))]);
        assert_eq!(
            f.registry
                .check_package_structure(std::slice::from_ref(&source)),
            Ok(())
        );
        assert_eq!(f.registry.check_storage_paths(&[source]), Ok(()));
    }
}

#[test]
fn nominal_and_fixed_array_roles_do_not_claim_payload_completeness() {
    let mut f = Fixture::new(&[]);
    let nominal = f.registry.declare_struct(&f.file, key("Forward")).unwrap();
    let target = CObjectType::structure(nominal.clone());
    let slot = local(&mut f, pointer_type(target), "owner");
    f.registry.register_local_owner(&slot).unwrap();
    assert_eq!(
        f.registry.members(&CAggregateRef::Struct(nominal)).unwrap(),
        None
    );
    let array = CObjectType::array(
        CObjectType::scalar(CScalarType::Int),
        CArrayLength::new(3).unwrap(),
    )
    .unwrap();
    let array_slot = local(&mut f, pointer_type(array), "array_owner");
    f.registry.register_local_owner(&array_slot).unwrap();
    assert_eq!(f.registry.owner_slots().count(), 2);
}

#[test]
fn owner_membership_cannot_cross_registry_function_or_duplicate_a_slot() {
    let mut f = Fixture::new(&[]);
    let mut foreign = Fixture::new(&[]);
    let ty = pointer_type(CObjectType::scalar(CScalarType::Int));
    let slot = local(&mut f, ty.clone(), "owner");
    let other_slot = local(&mut foreign, ty, "owner");
    let owner = f.registry.register_local_owner(&slot).unwrap();
    let other = foreign.registry.register_local_owner(&other_slot).unwrap();
    assert_eq!(
        f.registry.register_local_owner(&slot),
        Err(CRegistryError::DuplicateRegistration)
    );
    assert_eq!(
        f.registry.register_local_owner(&other_slot),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        f.registry.check_owner_slot(&f.function, &other),
        Err(CRegistryError::CrossRegistry)
    );
    let function = f
        .registry
        .register_function(
            &f.file,
            key("other"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    assert_eq!(
        f.registry.check_owner_slot(&function, &owner),
        Err(CRegistryError::WrongOwner)
    );
}

#[test]
fn canonical_owner_inventory_is_independent_of_brand_and_registration_order() {
    let inventories: Vec<_> = [false, true]
        .into_iter()
        .map(|reverse| {
            let mut f = Fixture::new(&[]);
            let names = if reverse { ["b", "a"] } else { ["a", "b"] };
            for name in names {
                let slot = local(
                    &mut f,
                    pointer_type(CObjectType::scalar(CScalarType::Int)),
                    name,
                );
                f.registry.register_local_owner(&slot).unwrap();
            }
            f.registry.inventory()
        })
        .collect();
    assert_eq!(inventories[0], inventories[1]);
}
