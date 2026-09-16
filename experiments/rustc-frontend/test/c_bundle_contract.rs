//! Violate private graph invariants only in a dedicated compiler test build.
use super::*;
use crate::output::bundle::preflight;

pub(super) fn check(graph: &CheckedGraph) {
    assert!(preflight(graph).is_ok());
    let mut missing_root = graph.clone();
    missing_root.crates.remove(&graph.root);
    assert!(preflight(&missing_root).is_err());
    let mut wrong_key = graph.clone();
    let (mut key, value) = wrong_key.crates.pop_first().unwrap();
    key.definition_path_hash ^= 1;
    wrong_key.crates.insert(key, value);
    assert!(preflight(&wrong_key).is_err());
    if graph.crates.len() > 1 {
        let mut wrong_manifest = graph.clone();
        let keys: Vec<_> = graph.crates.keys().copied().collect();
        let other = wrong_manifest.crates[&keys[1]].manifest.clone();
        wrong_manifest.crates.get_mut(&keys[0]).unwrap().manifest = other;
        assert!(preflight(&wrong_manifest).is_err());
    }
    for imported in graph
        .crates
        .values()
        .flat_map(|member| portable_backend_c::dialect::c_imported_constants(member.api.package()))
    {
        let owner = imported.dependency().package_identity().root();
        let mut missing = graph.clone();
        missing.crates.remove(&owner);
        assert!(preflight(&missing).is_err());
        let mut recertified = graph.clone();
        let member = recertified.crates.get_mut(&owner).unwrap();
        member.api = CDependencyApi::from_certificate(member.api.package().clone()).unwrap();
        assert!(
            preflight(&recertified)
                .unwrap_err()
                .contains("exact member certificate")
        );
    }
    let Some(imported) = graph
        .crates
        .values()
        .flat_map(|member| portable_backend_c::dialect::c_imported_functions(member.api.package()))
        .next()
    else {
        return;
    };
    let owner = imported.dependency().package_identity().root();
    let mut missing_owner = graph.clone();
    missing_owner.crates.remove(&owner);
    assert!(preflight(&missing_owner).is_err());
    let mut recertified = graph.clone();
    let member = recertified.crates.get_mut(&owner).unwrap();
    member.api = CDependencyApi::from_certificate(member.api.package().clone()).unwrap();
    assert!(
        preflight(&recertified)
            .unwrap_err()
            .contains("exact member certificate")
    );
    let mut wrong_manifest = graph.clone();
    let other = wrong_manifest
        .crates
        .iter()
        .find(|(id, _)| **id != owner)
        .unwrap()
        .1
        .manifest
        .clone();
    wrong_manifest.crates.get_mut(&owner).unwrap().manifest = other;
    assert!(preflight(&wrong_manifest).is_err());
}
