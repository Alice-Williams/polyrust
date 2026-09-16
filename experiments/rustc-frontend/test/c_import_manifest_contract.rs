//! Deliberate typed inventory corruptions against real compiler/C certificates.
use super::*;
use portable_backend_c::dialect::{c_defined_constants, c_defined_functions, c_imported_functions};

pub(crate) fn check(
    package: &RenderReadyPackage<CDialect>,
    manifest: &ApiManifest,
    owned: &BTreeMap<RustDeclarationId, CFunctionRef>,
    imported: &imports::ExpectedImports,
    constants: &constants::ExpectedConstants,
) {
    let rebuild = |expected: &imports::ExpectedImports| {
        ApiManifest::with_constants(
            package,
            manifest.exports.clone(),
            owned,
            expected,
            constants,
        )
    };
    assert_eq!(c_defined_constants(package).count(), constants.len());
    assert_eq!(c_defined_functions(package).count(), owned.len());
    assert_eq!(c_imported_functions(package).count(), imported.len());
    assert!(
        owned
            .keys()
            .all(|id| id.crate_id == manifest.exports.root.crate_id)
    );
    assert!(
        imported
            .keys()
            .all(|id| id.crate_id != manifest.exports.root.crate_id)
    );
    assert_eq!(&rebuild(imported).unwrap(), manifest);
    let Some((&id, (reference, proof))) = imported.first_key_value() else {
        assert!(manifest.canonical_json().is_ok());
        return;
    };
    assert!(
        manifest.canonical_json().is_err(),
        "standalone output omitted imports"
    );
    let mut missing = imported.clone();
    missing.remove(&id);
    assert!(rebuild(&missing).is_err());
    let mut extra = imported.clone();
    extra.insert(manifest.exports.root, (reference.clone(), proof.clone()));
    assert!(rebuild(&extra).is_err());
    let mut foreign_reference = imported.clone();
    foreign_reference.get_mut(&id).unwrap().0 = proof.function().clone();
    assert!(
        rebuild(&foreign_reference).is_err(),
        "owning handle substituted for consumer import"
    );
    let mut changed = manifest.clone();
    changed.imports.remove(&id);
    assert!(
        changed
            .verify_constants(
                package,
                manifest.exports.clone(),
                owned,
                imported,
                constants
            )
            .is_err()
    );
    if let Some((_, (_, other))) = imported.iter().find(|(other, _)| **other != id) {
        let mut wrong = imported.clone();
        wrong.get_mut(&id).unwrap().1 = other.clone();
        assert!(
            rebuild(&wrong).is_err(),
            "wrong owning declaration/certificate accepted"
        );
    }
}
