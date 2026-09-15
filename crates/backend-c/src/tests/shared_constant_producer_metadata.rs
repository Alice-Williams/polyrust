//! Target syntax validity does not authenticate arbitrary Rust API metadata.
use super::{
    CDependencyApi,
    constant_producer_tests::certify,
    owned_constant_fixture::{self, Shape},
    project_c_package,
};
use portable_codegen::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    MissingConstant,
    ExtraDeclaration,
    WrongNamespace,
    Private,
    Unreachable,
    ForeignDeclaration,
    WrongNode,
    DuplicateIdentity,
}

#[test]
fn constant_api_rejects_coordinated_but_false_source_metadata() {
    for mutation in [
        Mutation::MissingConstant,
        Mutation::ExtraDeclaration,
        Mutation::WrongNamespace,
        Mutation::Private,
        Mutation::Unreachable,
        Mutation::ForeignDeclaration,
        Mutation::WrongNode,
        Mutation::DuplicateIdentity,
    ] {
        let fixture = owned_constant_fixture::with_origin_changes(Shape::ConstantsOnly, |origin| {
            let graph = Arc::make_mut(&mut origin.crate_exports);
            let root = graph.root;
            let entries = graph.modules.get_mut(&root).unwrap();
            let name = RustExportName {
                namespace: RustExportNamespace::Value,
                name: "false_value".into(),
            };
            match mutation {
                Mutation::MissingConstant => {
                    entries.remove(&name).unwrap();
                }
                Mutation::ExtraDeclaration => {
                    entries.insert(
                        RustExportName {
                            name: "extra".into(),
                            ..name
                        },
                        RustExportTarget::Declaration(RustDeclarationId {
                            definition_path_hash: 999,
                            ..root
                        }),
                    );
                }
                Mutation::WrongNamespace => {
                    let target = entries.remove(&name).unwrap();
                    entries.insert(
                        RustExportName {
                            namespace: RustExportNamespace::Type,
                            ..name
                        },
                        target,
                    );
                }
                Mutation::Private => origin.visibility = RustVisibility::RestrictedTo(root),
                Mutation::Unreachable => origin.externally_reachable = false,
                Mutation::ForeignDeclaration => origin.declaration.crate_id = 999,
                Mutation::WrongNode => origin.node = RustSourceNode::Binding(0),
                Mutation::DuplicateIdentity => origin.declaration.definition_path_hash = 10,
            }
        });
        let projection_error = match mutation {
            Mutation::ForeignDeclaration => {
                Some("compiler export root disagrees with documentation ancestry")
            }
            Mutation::WrongNode => Some("documented declaration has body-local source provenance"),
            Mutation::DuplicateIdentity => {
                Some("source declaration maps to conflicting documentation owners")
            }
            _ => None,
        };
        if let Some(expected) = projection_error {
            let errors = project_c_package(fixture.registry, fixture.files).unwrap_err();
            assert!(
                errors.iter().any(|error| error.message.contains(expected)),
                "{mutation:?}: {errors:?}"
            );
            continue;
        }
        assert!(
            CDependencyApi::from_certificate(certify(&fixture)).is_err(),
            "{mutation:?}"
        );
    }
}

#[test]
fn constant_aliases_refer_to_one_original_declaration_witness() {
    let fixture = owned_constant_fixture::with_origin_changes(Shape::ConstantsOnly, |origin| {
        let graph = Arc::make_mut(&mut origin.crate_exports);
        let entries = graph.modules.get_mut(&graph.root).unwrap();
        let target = entries[&RustExportName {
            namespace: RustExportNamespace::Value,
            name: "false_value".into(),
        }];
        entries.insert(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: "false_alias".into(),
            },
            target,
        );
    });
    let api = CDependencyApi::from_certificate(certify(&fixture)).unwrap();
    assert_eq!(api.constants().count(), 8);
}
