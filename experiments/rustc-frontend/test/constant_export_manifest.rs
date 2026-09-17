//! Corrupt retained C alias evidence, including jointly changed export metadata.
use super::*;
use portable_codegen::RustExportName;

pub(crate) fn check(package: &RenderReadyPackage<CDialect>, manifest: &ApiManifest) {
    if manifest.foreign_constants.is_empty() {
        return;
    }
    let api =
        portable_backend_c::dialect::CDependencyApi::from_certificate(package.clone()).unwrap();
    manifest.verify_owner(&api).unwrap();
    assert!(manifest.canonical_json().is_err());
    let bytes = manifest.bundle_json().unwrap();
    assert!(bytes.starts_with("{\"schema_version\":6,"));
    assert!(bytes.len() <= manifest.bundle_bound().unwrap());
    for fault in 0..7 {
        let mut altered = manifest.clone();
        match fault {
            0 => altered.foreign_constants.clear(),
            1 => altered
                .foreign_constants
                .push(altered.foreign_constants[0].clone()),
            2 => altered.foreign_constants.reverse(),
            3 => {
                altered.constant_imports.clear();
            }
            4 => {
                let binding = altered.foreign_constants.remove(0);
                Arc::make_mut(&mut altered.exports)
                    .modules
                    .get_mut(&binding.module())
                    .unwrap()
                    .remove(binding.name());
            }
            5 => {
                let graph = Arc::make_mut(&mut altered.exports);
                let root = graph.root;
                graph.modules.get_mut(&root).unwrap().insert(
                    RustExportName {
                        namespace: RustExportNamespace::Type,
                        name: "extra_local_alias".into(),
                    },
                    RustExportTarget::Module(root),
                );
            }
            6 => {
                if altered.used_constant_imports.is_empty() {
                    altered.used_constant_imports = altered.constant_imports.clone();
                } else {
                    altered.used_constant_imports.clear();
                }
            }
            _ => unreachable!(),
        }
        assert!(
            altered.verify_owner(&api).is_err(),
            "alias manifest fault {fault}"
        );
    }
    println!("CONSTANT_EXPORT_MANIFEST\t7");
}
