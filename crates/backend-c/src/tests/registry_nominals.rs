//! Registry identity and exact nominal-definition inventory controls.

use crate::ast::{
    CAggregateRef, CDeclarationKey, CFileKey, CFileRef, CFileRole, CGeneratedOrigin, CIdentifier,
    CObjectType, CPointerTarget, CRegistry, CRegistryError, CScalarType, CSynthesisReason,
};
use portable_codegen::RelativeOutputPath;

pub(super) fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}

pub(super) fn registry() -> (CRegistry, CFileRef) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("src/generated.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    (registry, file)
}

fn scalar() -> CObjectType {
    CObjectType::scalar(CScalarType::I32)
}

#[test]
fn nominal_references_preserve_source_file_kind_and_exact_fields() {
    let (mut registry, file) = registry();
    let structure = registry.declare_struct(&file, key("Record")).unwrap();
    let union = registry.declare_union(&file, key("Payload")).unwrap();
    let enumeration = registry.declare_enum(&file, key("Status")).unwrap();
    assert_eq!(structure.key(), &key("Record"));
    assert_eq!(structure.file(), &file);
    assert_eq!(union.file(), &file);
    assert_eq!(enumeration.file(), &file);
    let owner = CAggregateRef::Struct(structure.clone());
    assert_eq!(registry.members(&owner).unwrap(), None);
    let field = registry
        .register_member(&owner, key("value"), scalar())
        .unwrap();
    assert_eq!(field.owner(), &owner);
    assert_eq!(field.ty(), &scalar());
    registry
        .define_aggregate(&owner, vec![field.clone()])
        .unwrap();
    assert_eq!(registry.members(&owner).unwrap(), Some([field].as_slice()));
    for ty in [
        CObjectType::structure(structure),
        CObjectType::union(union),
        CObjectType::enumeration(enumeration),
    ] {
        registry.check_type(&ty).unwrap();
    }
}

#[test]
fn foreign_registry_references_fail_even_with_identical_source_keys() {
    let (mut first, first_file) = registry();
    let (mut second, second_file) = registry();
    let local = first.declare_struct(&first_file, key("Record")).unwrap();
    let foreign = second.declare_struct(&second_file, key("Record")).unwrap();
    assert_eq!(local.key(), foreign.key());
    assert_ne!(local, foreign);
    assert_eq!(
        first.check_type(&CObjectType::structure(foreign.clone())),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        first.declare_union(&second_file, key("Wrong")),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        first.register_typedef(
            &first_file,
            key("Alias"),
            CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::structure(
                foreign
            ))))
        ),
        Err(CRegistryError::CrossRegistry)
    );
    let left_owner = CAggregateRef::Struct(local);
    let right = first.declare_struct(&first_file, key("Other")).unwrap();
    let right_owner = CAggregateRef::Struct(right);
    let field = first
        .register_member(&right_owner, key("value"), scalar())
        .unwrap();
    assert_eq!(
        first.check_member(&left_owner, &field),
        Err(CRegistryError::WrongOwner)
    );
}

#[test]
fn definitions_reject_empty_duplicate_missing_and_wrong_owner_members() {
    let (mut registry, file) = registry();
    let first = CAggregateRef::Struct(registry.declare_struct(&file, key("One")).unwrap());
    let second = CAggregateRef::Union(registry.declare_union(&file, key("Two")).unwrap());
    for owner in [&first, &second] {
        assert_eq!(
            registry.define_aggregate(owner, vec![]),
            Err(CRegistryError::EmptyDefinition)
        );
    }
    let a = registry
        .register_member(&first, key("a"), scalar())
        .unwrap();
    let b = registry
        .register_member(&first, key("b"), scalar())
        .unwrap();
    let foreign = registry
        .register_member(&second, key("a"), scalar())
        .unwrap();
    assert_eq!(
        registry.define_aggregate(&first, vec![foreign.clone()]),
        Err(CRegistryError::WrongOwner)
    );
    assert_eq!(
        registry.define_aggregate(&first, vec![a.clone()]),
        Err(CRegistryError::DefinitionInventoryMismatch)
    );
    assert_eq!(
        registry.define_aggregate(&first, vec![a.clone(), a.clone(), b.clone()]),
        Err(CRegistryError::DefinitionInventoryMismatch)
    );
    registry.define_aggregate(&first, vec![a, b]).unwrap();
    assert_eq!(
        registry.define_aggregate(&first, vec![]),
        Err(CRegistryError::AlreadyDefined)
    );
    assert_eq!(
        registry.register_member(&first, key("late"), scalar()),
        Err(CRegistryError::AlreadyDefined)
    );
    registry.define_aggregate(&second, vec![foreign]).unwrap();
}

#[test]
fn enumerators_have_enum_owners_int_values_and_exact_definition_inventory() {
    let (mut registry, file) = registry();
    let status = registry.declare_enum(&file, key("Status")).unwrap();
    let other = registry.declare_enum(&file, key("Other")).unwrap();
    assert_eq!(
        registry.define_enum(&status, vec![]),
        Err(CRegistryError::EmptyDefinition)
    );
    let min = registry
        .register_enumerator(&status, key("Minimum"), i32::MIN)
        .unwrap();
    let max = registry
        .register_enumerator(&status, key("Maximum"), i32::MAX)
        .unwrap();
    let wrong = registry
        .register_enumerator(&other, key("Wrong"), 0)
        .unwrap();
    assert_eq!(min.value(), i32::MIN);
    assert_eq!(max.value(), i32::MAX);
    assert_eq!(
        registry.define_enum(&status, vec![wrong]),
        Err(CRegistryError::WrongOwner)
    );
    assert_eq!(
        registry.define_enum(&status, vec![min.clone()]),
        Err(CRegistryError::DefinitionInventoryMismatch)
    );
    assert_eq!(
        registry.define_enum(&status, vec![min.clone(), min.clone(), max.clone()]),
        Err(CRegistryError::DefinitionInventoryMismatch)
    );
    registry
        .define_enum(&status, vec![min.clone(), max.clone()])
        .unwrap();
    assert_eq!(
        registry.enumerators(&status).unwrap(),
        Some([min, max].as_slice())
    );
    assert_eq!(
        registry.register_enumerator(&status, key("Late"), 1),
        Err(CRegistryError::AlreadyDefined)
    );
}

#[test]
fn recursive_nominal_pointers_do_not_require_recursive_alias_definitions() {
    let (mut registry, file) = registry();
    let node = registry.declare_struct(&file, key("Node")).unwrap();
    let owner = CAggregateRef::Struct(node.clone());
    let next_ty = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::structure(
        node.clone(),
    ))));
    let next = registry
        .register_member(&owner, key("next"), next_ty)
        .unwrap();
    registry.define_aggregate(&owner, vec![next]).unwrap();
    registry.check_type(&CObjectType::structure(node)).unwrap();
    // Completeness/layout/ownership certificates are explicitly later stages.
}

#[test]
fn duplicate_registration_is_not_a_second_identity_or_definition() {
    let (mut registry, file) = registry();
    registry.declare_struct(&file, key("Record")).unwrap();
    assert_eq!(
        registry.declare_struct(&file, key("Record")),
        Err(CRegistryError::DuplicateRegistration)
    );
    registry
        .register_typedef(&file, key("Alias"), scalar())
        .unwrap();
    assert_eq!(
        registry.register_typedef(&file, key("Alias"), CObjectType::scalar(CScalarType::U64)),
        Err(CRegistryError::DuplicateRegistration)
    );
    assert_eq!(
        registry.register_file(file.key().clone()),
        Err(CRegistryError::DuplicateRegistration)
    );
}

#[test]
fn canonical_file_inventory_does_not_contain_registry_addresses_or_counters() {
    let (mut a, _) = registry();
    let (mut b, _) = registry();
    let extra = [
        CFileKey {
            path: RelativeOutputPath::new("src/z.c").unwrap(),
            role: CFileRole::GeneratedSource,
        },
        CFileKey {
            path: RelativeOutputPath::new("src/a.c").unwrap(),
            role: CFileRole::RuntimeSource,
        },
    ];
    for file in extra.iter().cloned() {
        a.register_file(file).unwrap();
    }
    for file in extra.iter().rev().cloned() {
        b.register_file(file).unwrap();
    }
    assert_eq!(a.files().collect::<Vec<_>>(), b.files().collect::<Vec<_>>());
    assert_eq!(
        format!("{:?}", a.files().collect::<Vec<_>>()),
        format!("{:?}", b.files().collect::<Vec<_>>())
    );
}
