//! Descriptive library requirements are rederived, not trusted.
use super::*;
use portable_backend_c::dialect::{CDependencyApi, CSystemLibrary};

pub(crate) fn check(package: &RenderReadyPackage<CDialect>, manifest: &ApiManifest) {
    let expected = manifest
        .functions
        .iter()
        .map(|(id, function)| (*id, function.reference.clone()))
        .collect();
    let constants = manifest
        .constants
        .iter()
        .map(|(id, constant)| (*id, (constant.reference.clone(), constant.value)))
        .collect();
    let verify = |candidate: &ApiManifest| {
        candidate.verify_all_bindings(
            package,
            manifest.exports.clone(),
            &expected,
            &manifest.imports,
            &constants,
            &manifest.constant_imports,
        )
    };
    let api = CDependencyApi::from_certificate(package.clone()).unwrap();
    verify(manifest).unwrap();
    manifest.verify_owner(&api).unwrap();
    let mut changed = manifest.clone();
    let present = changed.system_libraries.remove(&CSystemLibrary::Math);
    if !present {
        changed.system_libraries.insert(CSystemLibrary::Math);
    }
    assert!(verify(&changed).is_err());
    assert!(changed.verify_owner(&api).is_err());
    let (with, without) = if present {
        (manifest, &changed)
    } else {
        (&changed, manifest)
    };
    assert_eq!(
        with.bundle_bound().unwrap() - without.bundle_bound().unwrap(),
        48
    );
    let json = manifest.canonical_json().unwrap();
    assert!(json.len() <= manifest.bundle_bound().unwrap());
    if present {
        assert!(json.contains("\"schema_version\":9"));
        assert!(json.contains("\"system_libraries\":[\"m\"]"));
        assert!(json.contains("\"return\":\"unit\""));
        assert!(json.contains("\"return\":\"bool\""));
    } else {
        assert!(!json.contains("system_libraries"));
    }
    eprintln!(
        "TRUNCATION_MANIFEST\t{}",
        if present { "missing" } else { "extra" }
    );
}
