//! Missing evidence and a proved empty prefix are distinct lattice states.
use super::*;
use crate::ast::{
    CConstness, CDeclarationKey, CGeneratedOrigin, CIdentifier, CScalarType, CSynthesisReason,
    numeric_fixture::Fixture,
};
fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}
#[test]
fn empty_coverage_is_vacuous_but_an_absent_predecessor_removes_the_claim() {
    let f = Fixture::new(&[]);
    let ty = CObjectType::scalar(CScalarType::Int);
    let absent = Prefixes::default();
    let mut empty = Prefixes::default();
    empty.insert(Bound::OriginalCount, None);
    let mut full = Prefixes::default();
    full.insert(Bound::OriginalCount, Some(Cell::Initialized));
    for joined in [
        absent.join(&empty, &ty, &f.registry).unwrap(),
        empty.join(&absent, &ty, &f.registry).unwrap(),
    ] {
        assert!(joined.get(&Bound::OriginalCount).is_none());
    }
    for joined in [
        empty.join(&full, &ty, &f.registry).unwrap(),
        full.join(&empty, &ty, &f.registry).unwrap(),
    ] {
        assert_eq!(
            joined.get(&Bound::OriginalCount),
            Some(&Some(Cell::Initialized))
        );
    }
}
#[test]
fn frontier_retirement_does_not_retarget_an_immutable_snapshot() {
    let mut f = Fixture::new(&[]);
    let frontier = f.local(CScalarType::Size, "frontier");
    let snapshot = f
        .registry
        .register_local(
            &f.scope,
            key("snapshot"),
            CObjectType::scalar(CScalarType::Size)
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap();
    let identity = f
        .registry
        .register_loop(&f.scope, key("construction"))
        .unwrap();
    let counter = Bound::Counter {
        identity: Box::new(identity),
        local: Box::new(frontier.clone()),
    };
    let snapshot_bound = Bound::Snapshot(Box::new(snapshot.clone()));
    let mut prefixes = Prefixes::default();
    prefixes.insert(counter.clone(), Some(Cell::Initialized));
    prefixes.insert(snapshot_bound.clone(), Some(Cell::Initialized));
    prefixes.invalidate_index(&Root::Local(frontier));
    assert!(prefixes.get(&counter).is_none());
    assert_eq!(
        prefixes.get(&snapshot_bound),
        Some(&Some(Cell::Initialized))
    );
    prefixes.invalidate_index(&Root::Local(snapshot));
    assert!(prefixes.get(&snapshot_bound).is_none());
}
