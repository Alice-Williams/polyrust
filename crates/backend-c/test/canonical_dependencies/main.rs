//! Exact typed-owner authority, without compiler-source admission.
mod closure;
mod fixture;
mod source;
use fixture::*;
use portable_backend_c::{ast::*, dialect::*};
use portable_codegen::TargetPackageOwner;

#[test]
fn type_api_retains_full_owner_without_inventing_source_exports() {
    let api = api(0);
    let expected = TargetPackageOwner::CanonicalInstance {
        instance: facts(0).instance().key(),
        profile: PROFILE,
    };
    assert_eq!(api.owner(), expected);
    assert_eq!(api.source_root(), None);
    assert!(api.functions().next().is_none());
    assert!(api.constants().next().is_none());
    assert!(api.foreign_constants().next().is_none());
    assert_eq!(api.structs().count(), 1);
    let proof = api.structs().next().unwrap();
    assert_eq!(proof.package_identity().owner(), expected);
    assert_eq!(proof.package_identity().source_root(), None);
    let descriptor = c_canonical_type_package(api.package()).unwrap();
    assert_eq!(descriptor.facts(), facts(0));
    assert_eq!(proof.record(), descriptor.record());
    assert_eq!(
        proof.members(),
        [descriptor.tag().clone(), descriptor.value().clone()]
    );
}

#[test]
fn distinct_instances_sharing_core_are_distinct_importable_owners() {
    let left = api(0);
    let right = api(100);
    assert_eq!(
        facts(0).instance().core_root(),
        facts(100).instance().core_root()
    );
    assert_ne!(left.owner(), right.owner());
    let mut registry = CRegistry::new();
    let left = registry
        .import_struct(left.structs().next().unwrap().clone())
        .unwrap();
    let right = registry
        .import_struct(right.structs().next().unwrap().clone())
        .unwrap();
    assert_ne!(left, right);
    assert_eq!(registry.imported_structs().count(), 2);
    assert_ne!(
        registry.imported_struct(&left).unwrap().package_identity(),
        registry.imported_struct(&right).unwrap().package_identity()
    );
}

#[test]
fn cloning_reuses_authority_but_recertifying_cannot_replace_it() {
    let api = api(0);
    let clone = api.clone();
    let original = api.structs().next().unwrap();
    assert_eq!(Some(original), clone.structs().next());
    let replacement = CDependencyApi::from_certificate(api.package().clone()).unwrap();
    let replaced = replacement.structs().next().unwrap();
    assert_eq!(api.owner(), replacement.owner());
    assert_ne!(original, replaced);
    let mut registry = CRegistry::new();
    let reference = registry.import_struct(original.clone()).unwrap();
    assert!(registry.import_struct(replaced.clone()).is_err());
    assert_eq!(registry.imported_struct(&reference).unwrap(), original);
    let mut independent = CRegistry::new();
    assert_eq!(
        independent
            .import_struct(clone.structs().next().unwrap().clone())
            .unwrap(),
        reference
    );
}

#[test]
fn foreign_type_proof_never_grants_declaration_ownership() {
    let api = api(0);
    let proof = api.structs().next().unwrap();
    let mut registry = CRegistry::new();
    let record = registry.import_struct(proof.clone()).unwrap();
    let owner = CAggregateRef::Struct(record);
    assert!(
        registry
            .define_aggregate(&owner, proof.members().to_vec())
            .is_err()
    );
    assert_eq!(registry.imported_structs().count(), 1);
}
