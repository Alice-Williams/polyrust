//! Coordinated snapshots cannot bypass the export graph consistency contract.
use super::*;

fn fixture_with_graph(
    change: impl FnOnce(&mut RustCrateExports),
) -> crate::dialect::shared::package_fixture::Fixture {
    let (mut public, mut helper) = origins();
    change(Arc::make_mut(&mut public.crate_exports));
    helper.crate_exports = public.crate_exports.clone();
    with_origins(
        CGeneratedOrigin::RustSource(Arc::new(public)),
        CGeneratedOrigin::RustSource(Arc::new(helper)),
    )
}

#[test]
fn coherent_reachable_cycles_and_unexpanded_foreign_edges_are_allowed() {
    let fixture = fixture_with_graph(|graph| {
        let root = graph.root;
        let entries = graph.modules.get_mut(&root).unwrap();
        for (name, target) in [
            ("cycle", root),
            (
                "foreign",
                RustDeclarationId {
                    crate_id: 8,
                    definition_path_hash: 1,
                },
            ),
        ] {
            entries.insert(
                RustExportName {
                    namespace: RustExportNamespace::Type,
                    name: name.into(),
                },
                RustExportTarget::Module(target),
            );
        }
    });
    lower_package(&fixture.files).unwrap();
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    RootMismatch,
    ForeignRoot,
    MissingRoot,
    OrphanModule,
    MissingReachableModule,
    ForeignModuleEntry,
    WrongModuleNamespace,
    MissingRootAncestry,
    MissingAliasOnlyAncestry,
    ExtraAncestry,
    WrongAncestryOwner,
}

#[test]
fn coordinated_malformed_export_snapshots_never_project() {
    for mutation in [
        Mutation::RootMismatch,
        Mutation::ForeignRoot,
        Mutation::MissingRoot,
        Mutation::OrphanModule,
        Mutation::MissingReachableModule,
        Mutation::ForeignModuleEntry,
        Mutation::WrongModuleNamespace,
        Mutation::MissingRootAncestry,
        Mutation::MissingAliasOnlyAncestry,
        Mutation::ExtraAncestry,
        Mutation::WrongAncestryOwner,
    ] {
        let fixture = fixture_with_graph(|graph| {
            let root = graph.root;
            let private = RustDeclarationId {
                definition_path_hash: 2,
                ..root
            };
            match mutation {
                Mutation::MissingRootAncestry => {
                    graph.module_ancestries.remove(&root);
                }
                Mutation::MissingAliasOnlyAncestry => {
                    let alias = add_alias_module(graph);
                    graph.module_ancestries.remove(&alias);
                }
                Mutation::ExtraAncestry => {
                    graph
                        .module_ancestries
                        .insert(private, graph.module_ancestries[&root].clone());
                }
                Mutation::WrongAncestryOwner => {
                    let alias = add_alias_module(graph);
                    graph
                        .module_ancestries
                        .insert(alias, graph.module_ancestries[&root].clone());
                }
                Mutation::RootMismatch => graph.root = private,
                Mutation::ForeignRoot => graph.root.crate_id = 8,
                Mutation::MissingRoot => {
                    graph.modules.remove(&root);
                }
                Mutation::OrphanModule => {
                    graph.modules.insert(private, BTreeMap::new());
                }
                Mutation::ForeignModuleEntry => {
                    graph.modules.insert(
                        RustDeclarationId {
                            crate_id: 8,
                            ..root
                        },
                        BTreeMap::new(),
                    );
                }
                Mutation::MissingReachableModule | Mutation::WrongModuleNamespace => {
                    graph.modules.get_mut(&root).unwrap().insert(
                        RustExportName {
                            namespace: if matches!(mutation, Mutation::WrongModuleNamespace) {
                                RustExportNamespace::Value
                            } else {
                                RustExportNamespace::Type
                            },
                            name: "module".into(),
                        },
                        RustExportTarget::Module(
                            if matches!(mutation, Mutation::WrongModuleNamespace) {
                                root
                            } else {
                                private
                            },
                        ),
                    );
                }
            }
        });
        assert!(lower_package(&fixture.files).is_err(), "{mutation:?}");
        assert!(
            crate::dialect::project_c_package(fixture.registry, fixture.files).is_err(),
            "{mutation:?}"
        );
    }
}

fn add_alias_module(graph: &mut RustCrateExports) -> RustDeclarationId {
    let root = graph.root;
    let alias = RustDeclarationId {
        definition_path_hash: 5,
        ..root
    };
    graph.modules.get_mut(&root).unwrap().insert(
        RustExportName {
            namespace: RustExportNamespace::Type,
            name: "api".into(),
        },
        RustExportTarget::Module(alias),
    );
    graph.modules.insert(
        alias,
        BTreeMap::from([(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: "invoke".into(),
            },
            RustExportTarget::Declaration(RustDeclarationId {
                definition_path_hash: 3,
                ..root
            }),
        )]),
    );
    let root_documentation = graph.module_ancestries[&root][0].clone();
    let module = Arc::new(RustModuleDocumentation {
        declaration: alias,
        parent: Some(root),
        location: root_documentation.location.clone(),
        documentation: vec!["alias-only public module".into()],
    });
    graph
        .module_ancestries
        .insert(alias, vec![root_documentation, module].into());
    alias
}

#[test]
fn alias_only_module_documentation_is_preserved_once_in_the_header() {
    let fixture = fixture_with_graph(|graph| {
        add_alias_module(graph);
    });
    let docs = lower_package(&fixture.files).unwrap();
    assert_eq!(
        docs[fixture.public.file()]
            .modules()
            .map(CComment::text)
            .collect::<Vec<_>>(),
        ["public root", "alias-only public module"],
    );
    assert_eq!(
        docs[fixture.helper.file()]
            .modules()
            .map(CComment::text)
            .collect::<Vec<_>>(),
        ["private ancestor"],
    );
}
