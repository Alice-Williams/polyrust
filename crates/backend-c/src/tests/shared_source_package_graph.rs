//! Graph, ancestry and documentation budgets apply without a declaration anchor.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    MissingRoot,
    ForeignRoot,
    UnreachableModule,
    MissingAncestry,
    WrongAncestryOwner,
    WrongParent,
    ForeignAncestor,
    Overdeep,
    TooManyBindings,
    TooManyModules,
    TooManyNameBytes,
    TooManyDocBytes,
}

#[test]
fn unanchored_metadata_cannot_bypass_graph_or_documentation_validation() {
    for mutation in [
        Mutation::MissingRoot,
        Mutation::ForeignRoot,
        Mutation::UnreachableModule,
        Mutation::MissingAncestry,
        Mutation::WrongAncestryOwner,
        Mutation::WrongParent,
        Mutation::ForeignAncestor,
        Mutation::Overdeep,
        Mutation::TooManyBindings,
        Mutation::TooManyModules,
        Mutation::TooManyNameBytes,
        Mutation::TooManyDocBytes,
    ] {
        let mut exports = super::super::package_source_fixture::origins()
            .0
            .crate_exports;
        let graph = Arc::make_mut(&mut exports);
        let root = graph.root;
        let other = RustDeclarationId {
            definition_path_hash: 55,
            ..root
        };
        match mutation {
            Mutation::MissingRoot => {
                graph.modules.remove(&root);
            }
            Mutation::ForeignRoot => {
                graph.root.crate_id += 1;
            }
            Mutation::UnreachableModule => {
                graph.modules.insert(other, Default::default());
            }
            Mutation::MissingAncestry => {
                graph.module_ancestries.clear();
            }
            Mutation::WrongAncestryOwner
            | Mutation::WrongParent
            | Mutation::ForeignAncestor
            | Mutation::TooManyDocBytes => {
                let mut ancestry = graph.module_ancestries[&root].to_vec();
                let module = Arc::make_mut(&mut ancestry[0]);
                match mutation {
                    Mutation::WrongAncestryOwner => module.declaration = other,
                    Mutation::WrongParent => module.parent = Some(other),
                    Mutation::ForeignAncestor => module.declaration.crate_id += 1,
                    Mutation::TooManyDocBytes => {
                        module.documentation = vec!["x".repeat(16 * 1024 * 1024 + 1)]
                    }
                    _ => unreachable!(),
                }
                graph.module_ancestries.insert(root, ancestry.into());
            }
            Mutation::Overdeep => {
                let module = graph.module_ancestries[&root][0].clone();
                graph
                    .module_ancestries
                    .insert(root, vec![module; 129].into());
            }
            Mutation::TooManyBindings => {
                let entries = graph.modules.get_mut(&root).unwrap();
                entries.clear();
                for index in 0..100_001 {
                    entries.insert(
                        RustExportName {
                            namespace: RustExportNamespace::Value,
                            name: format!("alias_{index}"),
                        },
                        RustExportTarget::Declaration(other),
                    );
                }
            }
            Mutation::TooManyModules => {
                for hash in 2..=100_001 {
                    graph.modules.insert(
                        RustDeclarationId {
                            definition_path_hash: hash,
                            ..root
                        },
                        Default::default(),
                    );
                }
            }
            Mutation::TooManyNameBytes => {
                graph.modules.get_mut(&root).unwrap().insert(
                    RustExportName {
                        namespace: RustExportNamespace::Value,
                        name: "a".repeat(16 * 1024 * 1024 + 1),
                    },
                    RustExportTarget::Declaration(other),
                );
            }
        }
        let (registry, files) = empty_package(exports);
        let result = documentation::lower_registered_package(registry.registrations(), &files);
        assert!(result.is_err(), "{mutation:?} was admitted");
    }
}

#[test]
fn finite_module_cycles_keep_one_documentation_attachment() {
    let mut exports = super::super::package_source_fixture::origins()
        .0
        .crate_exports;
    let graph = Arc::make_mut(&mut exports);
    let root = graph.root;
    let entries = graph.modules.get_mut(&root).unwrap();
    entries.clear();
    entries.insert(
        RustExportName {
            namespace: RustExportNamespace::Type,
            name: "cycle".into(),
        },
        RustExportTarget::Module(root),
    );
    let (registry, files) = empty_package(exports);
    let docs = documentation::lower_registered_package(registry.registrations(), &files).unwrap();
    assert_eq!(docs[files[0].identity()].modules().count(), 1);
    assert!(docs[files[1].identity()].modules().next().is_none());
}
