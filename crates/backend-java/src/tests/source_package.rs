//! Explicit crate provenance is independently checked, not inferred from a dummy field.
use crate::{
    ast::*,
    dialect::{JavaDependencyApi, JavaDialect},
};
use portable_codegen::*;
use std::sync::Arc;
#[path = "source_package_fixture.rs"]
mod fixture;
#[path = "source_package_native.rs"]
mod native;

#[test]
fn empty_facade_retains_explicit_graph_and_module_docs_without_api_authority() {
    let ready = fixture::certify(fixture::empty());
    let item = &ready.ast().files()[0].items()[0];
    assert_eq!(item.source_inventory.iter().len(), 1); // Synthesized facade only.
    assert_eq!(item.documentation.iter().count(), 2);
    let JavaFileItem::Type {
        source_package: Some(source),
        declaration,
        ..
    } = &item.item
    else {
        panic!("facade")
    };
    assert_eq!(source.exports(), &fixture::graph());
    assert_eq!(declaration.members.len(), 1); // Private constructor only.
    let output = render_certified_package(&crate::render::JavaRenderer, &ready).unwrap();
    assert_eq!(output.files().len(), 1);
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("text")
    };
    for marker in [
        "Empty facade root documentation.",
        "Nested alias module documentation.",
    ] {
        assert_eq!(text.matches(marker).count(), 1);
    }
    assert!(!text.contains("Runtime"));
    assert!(JavaDependencyApi::from_certificate(ready).is_err());
}

#[test]
fn explicit_package_preserves_existing_function_constant_and_record_apis() {
    use crate::tests::{source_constant_fixture::Fixture, source_dependency_fixture as functions};
    let packages = [
        functions::package(7, functions::functions(42)),
        Fixture::new(false).finish(),
        Fixture::new(true).finish(),
        functions::record_package(|_| {}),
    ];
    for package in packages {
        let exports = fixture::exports(&package);
        let before = fixture::certify(package.clone());
        let explicit = fixture::attach(package.clone(), exports.clone());
        let after = fixture::certify(explicit);
        assert_eq!(
            render_certified_package(&crate::render::JavaRenderer, &before).unwrap(),
            render_certified_package(&crate::render::JavaRenderer, &after).unwrap()
        );
        let original = JavaDependencyApi::from_certificate(before).unwrap();
        let api = JavaDependencyApi::from_certificate(after).unwrap();
        assert_eq!(api.root(), original.root());
        assert_eq!(api.functions().count(), original.functions().count());
        assert_eq!(api.constants().count(), original.constants().count());
        fixture::certify(fixture::attach(package, Arc::new((*exports).clone())));
    }
}

#[derive(Clone, Copy, Debug)]
enum Fault {
    Namespace,
    Path,
    Role,
    Placement,
    Duplicate,
    Name,
    Kind,
    Unregistered,
    NoGraph,
    BrokenAncestry,
}
#[test]
fn malformed_explicit_package_is_rejected_before_linking() {
    for fault in [
        Fault::Namespace,
        Fault::Path,
        Fault::Role,
        Fault::Placement,
        Fault::Duplicate,
        Fault::Name,
        Fault::Kind,
        Fault::Unregistered,
        Fault::NoGraph,
        Fault::BrokenAncestry,
    ] {
        let package = fixture::rebuild(fixture::empty(), |file| match fault {
            Fault::Namespace => file.module = JavaPackage::RustCrate(8),
            Fault::Path => {
                file.path = RelativeOutputPath::new(
                    "src/main/java/org/polyrust/generated/r0000000000000007/Other.java",
                )
                .unwrap()
            }
            Fault::Role => file.role = SourceRole::Implementation,
            Fault::Placement => file.placement = JavaFilePlacement::NativeTest,
            Fault::Duplicate => file.items.push(file.items[0].clone()),
            _ => {
                let JavaFileItem::Type {
                    source_package,
                    declaration,
                    ..
                } = &mut file.items[0]
                else {
                    panic!("facade")
                };
                match fault {
                    Fault::Name => declaration.name = JavaIdentifier::new("Other").unwrap(),
                    Fault::Kind => declaration.kind = JavaDeclarationKind::Interface,
                    Fault::Unregistered => declaration.declared = None,
                    Fault::NoGraph | Fault::BrokenAncestry => {
                        let mut graph = (*fixture::graph()).clone();
                        if matches!(fault, Fault::NoGraph) {
                            graph.modules.clear();
                        } else {
                            graph.module_ancestries.clear();
                        }
                        *source_package = Some(JavaSourcePackage::new(Arc::new(graph)));
                    }
                    _ => unreachable!(),
                }
            }
        });
        assert!(
            verify_unresolved_package(&JavaDialect, package).is_err(),
            "{fault:?}"
        );
    }
}

#[test]
fn explicit_graph_cannot_replace_owned_declaration_or_record_field_provenance() {
    use crate::tests::source_dependency_fixture as functions;
    for package in [
        functions::package(7, functions::functions(42)),
        functions::record_package(|_| {}),
    ] {
        let mut graph = (*fixture::exports(&package)).clone();
        graph.modules.get_mut(&graph.root).unwrap().insert(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: "forged".into(),
            },
            RustExportTarget::Declaration(RustDeclarationId {
                definition_path_hash: 500,
                ..graph.root
            }),
        );
        assert!(
            verify_unresolved_package(&JavaDialect, fixture::attach(package, Arc::new(graph)))
                .is_err()
        );
    }
}

#[test]
fn linked_metadata_and_coupled_docs_are_not_the_original_exact_rewrite() {
    let original = fixture::certify(fixture::empty());
    let mut graph = (*fixture::graph()).clone();
    let path = graph.module_ancestries.get_mut(&graph.root).unwrap();
    let mut root = (**path.first().unwrap()).clone();
    root.documentation = vec!["Changed root docs.".into()];
    let root = Arc::new(root);
    for path in graph.module_ancestries.values_mut() {
        Arc::make_mut(path)[0] = root.clone();
    }
    let replacement = fixture::certify(fixture::attach(fixture::empty(), Arc::new(graph)));
    let expected = &original.ast().files()[0].items()[0];
    let changed = &replacement.ast().files()[0].items()[0];
    for metadata in [false, true] {
        for docs in [false, true] {
            let mut item = expected.clone();
            if metadata {
                item.item = changed.item.clone();
            }
            if docs {
                item.documentation = changed.documentation.clone();
            }
            // Safe APIs expose no mutable LinkedFile slot. The shared verifier
            // rederives the original item and compares this exact whole value.
            assert_eq!(item == *expected, !metadata && !docs);
        }
    }
    verify_linked_package(original.ast()).unwrap();
    verify_linked_package(replacement.ast()).unwrap();
}

#[test]
fn source_package_requires_real_facade_registration_not_just_its_name() {
    let package = fixture::empty_with_origin(SynthesisReason::TestHarness);
    let errors = verify_unresolved_package(&JavaDialect, package).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("PackageEntryPoint facade"))
    );
}

#[test]
fn declaration_free_package_does_not_bypass_export_graph_resource_limits() {
    let mut graph = (*fixture::graph()).clone();
    let root = graph.root;
    for index in 0..99_999 {
        graph.modules.get_mut(&root).unwrap().insert(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: format!("alias{index}"),
            },
            RustExportTarget::Declaration(RustDeclarationId {
                definition_path_hash: 3,
                ..root
            }),
        );
    }
    // 99,999 value aliases plus the two module-alias edges exceed 100,000.
    let errors = verify_unresolved_package(
        &JavaDialect,
        fixture::attach(fixture::empty(), Arc::new(graph)),
    )
    .unwrap_err();
    assert!(errors.iter().any(|error| error.code
        == portable_diagnostics::DiagnosticCode::TargetResourceLimit
        && error.message.contains("export bindings")));
}

#[test]
fn explicit_package_checks_record_field_only_provenance_mismatches() {
    use crate::tests::source_dependency_fixture as functions;
    let package = functions::record_package(|fixture| {
        let JavaRecordComponentOrigin::RustSource(field) =
            &mut fixture.record.record_components[0].origin
        else {
            panic!("source field")
        };
        let origin = Arc::make_mut(&mut field.origin);
        let graph = Arc::make_mut(&mut origin.crate_exports);
        graph.modules.get_mut(&graph.root).unwrap().insert(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: "field_only".into(),
            },
            RustExportTarget::Declaration(RustDeclarationId {
                definition_path_hash: 99,
                ..graph.root
            }),
        );
    });
    let exports = fixture::exports(&package);
    let errors =
        verify_unresolved_package(&JavaDialect, fixture::attach(package, exports)).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("conflicting export inventories"))
    );
}
