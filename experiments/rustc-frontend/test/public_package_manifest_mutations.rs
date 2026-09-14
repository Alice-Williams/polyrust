//! Typed metadata mutations are rejected against immutable compiler/target inputs.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    MissingAlias,
    RetargetAlias,
    ForeignRoot,
    MissingFunction,
    WidenPrivate,
    WrongReference,
    WrongName,
    WrongDefinitionFile,
    WrongHeader,
}

pub(crate) fn check(
    package: &RenderReadyPackage<CDialect>,
    manifest: &ApiManifest,
    exports: &Arc<RustCrateExports>,
    expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
) {
    manifest.verify(package, exports.clone(), expected).unwrap();
    let public = manifest
        .functions
        .iter()
        .find(|(_, value)| value.linkage == CLinkage::External)
        .unwrap()
        .0
        .to_owned();
    let private = manifest
        .functions
        .iter()
        .find(|(_, value)| value.linkage == CLinkage::Internal)
        .unwrap()
        .0
        .to_owned();
    for mutation in [
        Mutation::MissingAlias,
        Mutation::RetargetAlias,
        Mutation::ForeignRoot,
        Mutation::MissingFunction,
        Mutation::WidenPrivate,
        Mutation::WrongReference,
        Mutation::WrongName,
        Mutation::WrongDefinitionFile,
        Mutation::WrongHeader,
    ] {
        let mut changed = manifest.clone();
        match mutation {
            Mutation::MissingAlias | Mutation::RetargetAlias => {
                let graph = Arc::make_mut(&mut changed.exports);
                let root = graph.modules.get_mut(&graph.root).unwrap();
                let key = root.keys().find(|key| key.name == "alias").unwrap().clone();
                if matches!(mutation, Mutation::MissingAlias) {
                    root.remove(&key);
                } else {
                    root.insert(key, RustExportTarget::Declaration(private));
                }
            }
            Mutation::ForeignRoot => Arc::make_mut(&mut changed.exports).root.crate_id ^= 1,
            Mutation::MissingFunction => {
                changed.functions.remove(&private);
            }
            Mutation::WidenPrivate => {
                changed.functions.get_mut(&private).unwrap().linkage = CLinkage::External
            }
            Mutation::WrongReference => {
                let wrong = changed.functions[&private].reference.clone();
                changed.functions.get_mut(&public).unwrap().reference = wrong;
            }
            Mutation::WrongName => {
                changed.functions.get_mut(&public).unwrap().name =
                    CIdentifier::new("plausible_but_wrong").unwrap()
            }
            Mutation::WrongDefinitionFile => {
                changed.functions.get_mut(&private).unwrap().implementation = changed.header.clone()
            }
            Mutation::WrongHeader => changed.header = changed.implementation.clone(),
        }
        assert!(
            changed.verify(package, exports.clone(), expected).is_err(),
            "{mutation:?}"
        );
    }
    let mut incomplete = expected.clone();
    incomplete.remove(&private);
    assert!(ApiManifest::new(package, exports.clone(), &incomplete).is_err());
    let mut extra = expected.clone();
    extra.insert(exports.root, expected[&private].clone());
    assert!(ApiManifest::new(package, exports.clone(), &extra).is_err());
    let mut oversized = manifest.clone();
    let graph = Arc::make_mut(&mut oversized.exports);
    graph.modules.get_mut(&graph.root).unwrap().insert(
        portable_codegen::RustExportName {
            namespace: RustExportNamespace::Value,
            name: "x".repeat(8 * 1024 * 1024 / 6),
        },
        RustExportTarget::Declaration(public),
    );
    assert!(oversized.canonical_json().unwrap_err().contains("8 MiB"));
}
