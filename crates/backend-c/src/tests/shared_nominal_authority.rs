//! Attack private summaries to prove they cannot replace original certificates.
use crate::{ast::*, dialect::shared::nominal_fixture};

#[test]
fn renamed_reordered_truncated_and_foreign_layout_summaries_reject_atomically() {
    let producer = nominal_fixture::producer();
    let original = producer.structs().next().unwrap();
    let another = nominal_fixture::producer();
    let other = another.structs().next().unwrap();
    let mut variants = vec![];
    let mut forged = original.clone();
    forged.export.symbol = CIdentifier::new("forged_tag").unwrap();
    variants.push(forged);
    let mut forged = original.clone();
    forged.export.members.reverse();
    variants.push(forged);
    let mut forged = original.clone();
    forged.export.members.pop();
    variants.push(forged);
    let mut forged = original.clone();
    forged.export.names.insert(
        original.members()[0].clone(),
        CIdentifier::new("forged_field").unwrap(),
    );
    variants.push(forged);
    let mut forged = original.clone();
    forged.export.names.clear();
    variants.push(forged);
    let mut forged = original.clone();
    forged.export = other.export.clone();
    variants.push(forged);
    let mut forged = original.clone();
    forged.authority = other.authority.clone();
    variants.push(forged);
    for proof in variants {
        assert!(proof.authenticate().is_err());
        let mut registry = CRegistry::new();
        let before = format!("{registry:?}");
        assert!(registry.import_struct(proof).is_err());
        assert_eq!(format!("{registry:?}"), before);
    }
}

#[test]
fn identical_spelling_and_shape_do_not_establish_nominal_identity() {
    let first = nominal_fixture::producer();
    let second = nominal_fixture::producer();
    let a = first.structs().next().unwrap();
    let b = second.structs().next().unwrap();
    assert_eq!(a.symbol(), b.symbol());
    assert_ne!(a, b);
    assert_ne!(a.record(), b.record());
    let mut registry = CRegistry::new();
    registry.import_struct(a.clone()).unwrap();
    assert!(registry.imported_struct(b.record()).is_err());
    let before = format!("{registry:?}");
    assert!(registry.import_struct(b.clone()).is_err());
    assert_eq!(format!("{registry:?}"), before);
    assert!(
        registry
            .check_member(&CAggregateRef::Struct(a.record().clone()), &b.members()[0])
            .is_err()
    );
}

#[test]
fn type_only_imports_remeasure_original_resource_evidence() {
    let producer = nominal_fixture::producer();
    let original = producer.structs().next().unwrap();
    let actual = original.authority.stack_bound_bytes;
    assert!(actual > 0);
    for cost in [0, actual - 1, actual + 1, u64::MAX] {
        let mut forged = original.clone();
        std::sync::Arc::make_mut(&mut forged.authority).stack_bound_bytes = cost;
        // The type and all its fields are authentic. Only cached cost is false.
        forged.authenticate().unwrap();
        let input = nominal_fixture::fixture(140, &[forged], &[nominal_fixture::Operation::Copy]);
        let error = nominal_fixture::certify(input).unwrap_err();
        assert!(error.contains("stack evidence differs"), "{cost}: {error}");
    }
}
