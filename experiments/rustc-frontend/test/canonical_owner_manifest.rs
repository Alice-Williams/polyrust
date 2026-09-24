//! Direct source-only manifest boundary, before canonical graph publication.
#[allow(
    dead_code,
    reason = "This focused boundary probe does not lower compiler inputs"
)]
#[path = "../src/api_manifest/mod.rs"]
mod api_manifest;
#[path = "../../../crates/backend-c/test/canonical_dependencies/fixture.rs"]
mod fixture;
#[allow(
    dead_code,
    reason = "The shared fixture also supports non-manifest owner tests"
)]
#[path = "../../../crates/backend-c/test/canonical_dependencies/source.rs"]
mod source;
use api_manifest::ApiManifest;
use portable_backend_c::{ast::*, dialect::*};
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

fn manifest(api: &CDependencyApi) -> Result<ApiManifest, String> {
    let function = api.functions().next().unwrap();
    let CGeneratedOrigin::RustSource(origin) = &function.function().key().origin else {
        panic!("source fixture");
    };
    ApiManifest::with_all_bindings(
        api.package(),
        origin.crate_exports.clone(),
        &BTreeMap::from([(function.declaration(), function.function().clone())]),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
}

#[test]
fn unused_canonical_import_is_rejected_before_any_source_manifest_is_produced() {
    let root = source::root(70);
    let stem = format!("polyrust_{:016x}", root.crate_id);
    let plain = source::source_named(root, &[], &[], false, &stem).unwrap();
    let accepted = manifest(&plain).unwrap();
    accepted.verify_owner(&plain).unwrap();
    assert!(!accepted.bundle_json().unwrap().is_empty());

    let canonical = fixture::api(0);
    let imported = source::source_named(
        root,
        &[canonical.structs().next().unwrap().clone()],
        &[],
        false,
        &stem,
    )
    .unwrap();
    assert!(imported.structs().next().is_none());
    assert_eq!(imported.dependencies().unwrap().len(), 1);
    assert_eq!(imported.source_root(), Some(root));
    let expected = "C source manifest does not yet support canonical type dependencies";
    assert_eq!(manifest(&imported).unwrap_err(), expected);
    assert_eq!(accepted.verify_owner(&imported).unwrap_err(), expected);
}

#[test]
fn canonical_package_cannot_masquerade_as_its_core_source_root() {
    let canonical = fixture::api(0);
    let root = fixture::facts(0).instance().core_root();
    let forged = Arc::new(RustCrateExports {
        root,
        modules: BTreeMap::new(),
        module_ancestries: BTreeMap::new(),
    });
    let result = ApiManifest::with_all_bindings(
        canonical.package(),
        forged,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    assert_eq!(
        result.unwrap_err(),
        "C source manifest does not support canonical type owners"
    );
}
