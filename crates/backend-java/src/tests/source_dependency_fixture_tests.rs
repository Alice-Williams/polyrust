//! Independent preparation for owner-API tests; no foreign-call support is claimed.
use super::source_dependency_fixture::*;
use crate::{ast::*, dialect::JavaStructuralRenderer};
use portable_codegen::{OutputContents, render_certified_package};

#[test]
fn source_api_fixture_certifies_scalars_arities_privacy_and_alias_metadata() {
    let draft = package(7, functions(42));
    assert_eq!(draft.callables().len(), 4);
    let certificate = certify(draft);
    let output = render_certified_package(&JavaStructuralRenderer, &certificate).unwrap();
    let [file] = output.files() else {
        panic!("one source file")
    };
    let OutputContents::Text(text) = file.contents() else {
        panic!("Java source")
    };
    assert!(text.contains("private Generated()"));
    assert!(text.contains("public static int fn000000000000000a()"));
    assert!(text.contains("public static int fn000000000000000b(final int p0, final boolean p1, final int p2, final boolean p3)"));
    assert!(text.contains("public static boolean fn000000000000000c(final boolean p0)"));
    assert!(text.contains("private static int fn000000000000000d(final int p0)"));
    assert_eq!(text.matches("Source owner documentation.").count(), 1);
    for hash in 10..=13 {
        assert_eq!(
            text.matches(&format!("Function {hash} documentation."))
                .count(),
            1
        );
    }
    for _ in 0..2 {
        assert_eq!(
            output,
            render_certified_package(&JavaStructuralRenderer, &certificate).unwrap()
        );
    }
}

#[test]
fn same_declaration_ids_can_have_different_certified_bodies() {
    let first = certify(package(7, functions(42)));
    let second = certify(package(7, functions(-7)));
    assert_ne!(
        render_certified_package(&JavaStructuralRenderer, &first).unwrap(),
        render_certified_package(&JavaStructuralRenderer, &second).unwrap()
    );
    let item = &first.ast().files()[0].items()[0].item;
    let JavaFileItem::Type { declaration, .. } = item else {
        panic!("source facade")
    };
    assert_eq!(
        declaration
            .members
            .iter()
            .filter(|member| matches!(member, JavaMember::Method(_)))
            .count(),
        4
    );
}
