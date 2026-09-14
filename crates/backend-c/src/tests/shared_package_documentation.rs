//! Public items behind private Rust ancestry must not publish private module docs.
use super::*;
use crate::dialect::shared::package_fixture::with_origins;
use portable_codegen::*;

#[path = "shared_package_export_coherence.rs"]
mod export_coherence;

use crate::dialect::shared::package_source_fixture::origins;

#[test]
fn public_function_docs_and_private_ancestry_are_routed_to_different_files_once() {
    let (public, helper) = origins();
    let mut fixture = with_origins(
        CGeneratedOrigin::RustSource(Arc::new(public)),
        CGeneratedOrigin::RustSource(Arc::new(helper)),
    );
    let docs = lower_package(&fixture.files).unwrap();
    let header = &docs[fixture.public.file()];
    let source = &docs[fixture.helper.file()];
    assert_eq!(
        header.modules().map(CComment::text).collect::<Vec<_>>(),
        ["public root"]
    );
    assert_eq!(
        source.modules().map(CComment::text).collect::<Vec<_>>(),
        ["private ancestor"]
    );
    assert_eq!(
        header
            .comments(&CDocumentationOwner::Function(fixture.public.clone()))
            .iter()
            .map(CComment::text)
            .collect::<Vec<_>>(),
        ["public operation"]
    );
    assert_eq!(
        source
            .comments(&CDocumentationOwner::Function(fixture.helper.clone()))
            .iter()
            .map(CComment::text)
            .collect::<Vec<_>>(),
        ["private helper"]
    );
    assert_eq!(header.all().count(), 2);
    assert_eq!(source.all().count(), 2);
    fixture.files.reverse();
    assert_eq!(docs, lower_package(&fixture.files).unwrap());
}

#[test]
fn cross_file_export_module_and_declaration_conflicts_are_rejected() {
    for mutation in 0..3 {
        let (public, mut helper) = origins();
        let expected = match mutation {
            0 => {
                Arc::make_mut(&mut helper.crate_exports)
                    .modules
                    .insert(helper.module, BTreeMap::new());
                "conflicting compiler export metadata"
            }
            1 => {
                Arc::make_mut(&mut Arc::make_mut(&mut helper.module_ancestors)[1])
                    .documentation
                    .push("changed ancestry".into());
                "conflicting documentation for one source module"
            }
            2 => {
                helper.declaration = public.declaration;
                "conflicting documentation owners"
            }
            _ => unreachable!(),
        };
        let fixture = with_origins(
            CGeneratedOrigin::RustSource(Arc::new(public)),
            CGeneratedOrigin::RustSource(Arc::new(helper)),
        );
        assert!(
            lower_package(&fixture.files)
                .unwrap_err()
                .contains(expected)
        );
    }
}

#[test]
fn certified_pair_renders_public_and_private_documentation_on_only_their_own_side() {
    use crate::dialect::shared::{CDialect, CStructuralRenderer, project_c_package};
    let (public, helper) = origins();
    let fixture = with_origins(
        CGeneratedOrigin::RustSource(Arc::new(public)),
        CGeneratedOrigin::RustSource(Arc::new(helper)),
    );
    let package = project_c_package(fixture.registry, fixture.files).unwrap();
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("C text")
        };
        for (comment, header_owned) in [
            ("public root", true),
            ("public operation", true),
            ("private ancestor", false),
            ("private helper", false),
        ] {
            assert_eq!(
                text.matches(&format!("/* {comment} */")).count(),
                usize::from(file.path().ends_with(".h") == header_owned)
            );
        }
    }
}
