//! Owned public result types, field identities and the deliberately closed import boundary.
use super::{
    CDependencyApi, CDialect, CStructuralRenderer, project_c_package,
    result_fixture::{Mutation, PublicApi, public_fixture},
};
use portable_codegen::*;

#[path = "shared_public_results_native.rs"]
mod native;
#[path = "shared_public_result_scopes.rs"]
mod scopes;

fn linked(mode: Mutation, api: PublicApi) -> Result<LinkedTargetPackage<CDialect>, String> {
    let (registry, files) = public_fixture(mode, api);
    let ast = project_c_package(registry, files).map_err(|e| format!("{e:?}"))?;
    let checked = verify_unresolved_package(&CDialect, ast).map_err(|e| format!("{e:?}"))?;
    TargetLinker::new(CDialect)
        .link_ast(&checked)
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn public_result_header_owns_types_fields_prototypes_and_dynamic_imports() {
    let linked = linked(Mutation::None, PublicApi::ResultSignatures).unwrap();
    let header = linked
        .files()
        .iter()
        .find(|f| f.module().key().path.as_str().ends_with(".h"))
        .unwrap();
    let source = linked
        .files()
        .iter()
        .find(|f| f.module().key().path.as_str().ends_with(".c"))
        .unwrap();
    assert!(header.file_imports().is_empty());
    assert_eq!(source.file_imports().len(), 1);
    assert_eq!(source.file_imports()[0].destination(), header.file());
    let h = &header.items()[0];
    let s = &source.items()[0];
    assert_eq!(h.spelling.types, s.spelling.types);
    let fields: Vec<_> = h
        .spelling
        .values
        .iter()
        .filter(|(v, _)| matches!(v, super::bindings::CValueBinding::Member(_)))
        .collect();
    assert_eq!(fields.len(), 2);
    for (field, spelling) in fields {
        assert_eq!(&s.spelling.values[field], spelling);
    }
    let measured = super::resources::measure_package(&linked).unwrap();
    assert_eq!(measured.total.automatic_bytes, 60);
    assert_eq!(measured.total.function_frames.len(), 4);
    assert!(measured.files[header.module()].function_frames.is_empty());
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    let mut bytes = 0;
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        bytes += text.len() as u64;
        assert_eq!(
            text.matches("struct poly_result {").count(),
            usize::from(file.path().ends_with(".h"))
        );
        assert!(!text.contains("runtime") && !text.contains("malloc"));
    }
    assert!(bytes <= measured.total.source_bound);
    assert_eq!(
        output,
        render_certified_package(&CStructuralRenderer, &certified).unwrap()
    );
}

#[test]
fn public_result_cannot_publish_unchecked_nominal_dependency_metadata() {
    for api in [PublicApi::ResultSignatures, PublicApi::ScalarOnly] {
        let certified =
            certify_resolved_package(&CDialect, linked(Mutation::None, api).unwrap()).unwrap();
        let error = CDependencyApi::from_certificate(certified).unwrap_err();
        assert!(
            format!("{error:?}")
                .contains("aggregate-bearing headers require certified nominal imports")
        );
    }
}

#[test]
fn public_result_preserves_order_initialization_and_closed_call_rules() {
    for (mode, diagnostic) in [
        (Mutation::Public, "source-private"),
        (
            Mutation::LateDeclaration,
            "record declarations before signatures",
        ),
        (Mutation::Uninitialized, "initialized locals"),
        (Mutation::Recursive, "call"),
    ] {
        let error = match linked(mode, PublicApi::ResultSignatures) {
            Err(error) => error,
            Ok(package) => format!(
                "{:?}",
                certify_resolved_package(&CDialect, package).unwrap_err()
            ),
        };
        assert!(
            error.to_lowercase().contains(diagnostic),
            "{mode:?}: {error}"
        );
    }
}
