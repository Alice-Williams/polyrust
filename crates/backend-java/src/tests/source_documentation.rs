use super::*;
use crate::ast::*;
use crate::tests::source_documentation_fixture as fixture;
use portable_codegen::*;
#[path = "source_documentation/native.rs"]
mod native;
use crate::tests::source_record_fixture as records;

#[test]
fn checked_source_documents_render_at_exact_owners_in_attribute_order() {
    let text = fixture::text(fixture::package());
    for marker in [
        "crate docs first",
        "private module documentation",
        "alias-only module documentation",
        "record first",
        "record second",
        "inspect first",
        "inspect second",
        "value field",
        "flag field",
    ] {
        assert_eq!(text.matches(marker).count(), 1, "{marker}: {text}");
    }
    let at = |marker: &str| text.find(marker).unwrap();
    assert!(at("private module documentation") < at("crate docs first"));
    assert!(at("private module documentation") < at("alias-only module documentation"));
    assert!(at("crate docs first") < at("public final class Generated"));
    assert!(at("inspect first") < at("inspect second"));
    assert!(at("inspect second") < at("public static int inspect"));
    assert!(at("record first") < at("record second"));
    assert!(at("record second") < at("private record Cell("));
    let record = &text[at("private record Cell(")..];
    assert!(record.find("value field").unwrap() < record.find("int value,").unwrap());
    assert!(record.find("flag field").unwrap() < record.find("boolean flag").unwrap());
    assert!(!text.contains(fixture::HOSTILE));
    assert!(!text.contains(r"\u002a"));
    assert_eq!(
        text.matches(JavaDocComment::new(fixture::HOSTILE).text())
            .count(),
        3
    );
}

#[test]
fn module_documentation_requires_an_emitted_facade_not_a_similarly_named_type() {
    let package = fixture::fixture(records::Facade::Harness).finish();
    let errors = verify_unresolved_package(&JavaDialect, package).unwrap_err();
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("exactly one emitted PackageEntryPoint")),
        "{errors:?}"
    );
}

#[test]
fn linked_attachments_are_exactly_rederived_and_mutations_change_item_equality() {
    let package = fixture::package();
    let verified = verify_unresolved_package(&JavaDialect, package.clone()).unwrap();
    let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
    verify_linked_package(&linked).unwrap();
    let item = &linked.files()[0].items()[0];
    assert_eq!(item.documentation, lower(&package, &item.item).unwrap());
    assert_eq!(item.documentation.iter().count(), 7);
    for mutation in 0..7 {
        let mut tampered = item.clone();
        let key = *tampered.documentation.attachments.keys().next().unwrap();
        match mutation {
            0 => {
                tampered.documentation.attachments.remove(&key);
            }
            1 => {
                tampered
                    .documentation
                    .attachments
                    .get_mut(&key)
                    .unwrap()
                    .comments
                    .push(JavaDocComment::new("forged"));
            }
            2 => {
                tampered
                    .documentation
                    .attachments
                    .get_mut(&key)
                    .unwrap()
                    .indentation += 1;
            }
            3 => {
                tampered.documentation.root = None;
            }
            4 => {
                tampered.documentation.module_order.clear();
            }
            5 => {
                let attachment = tampered.documentation.attachments[&key].clone();
                tampered
                    .documentation
                    .attachments
                    .insert(JavaDocumentationOwner::Module(records::id(99)), attachment);
            }
            6 => {
                tampered
                    .documentation
                    .attachments
                    .get_mut(&key)
                    .unwrap()
                    .comments
                    .reverse();
            }
            _ => unreachable!(),
        }
        // This is the exact equality compared by the shared post-link rederivation.
        // LinkedFile itself exposes no mutable item slot to safe external callers.
        assert_ne!(tampered, *item);
        assert_ne!(
            tampered.documentation,
            lower(&package, &tampered.item).unwrap()
        );
    }
    certify_resolved_package(&JavaDialect, linked).unwrap();
}

#[test]
fn shared_module_docs_stay_in_the_facade_file_and_other_declarations_keep_their_own() {
    let mut fixture = fixture::fixture(records::Facade::EntryPoint);
    let mut origin = fixture::metadata();
    origin.declaration = records::id(11);
    origin.documentation = vec!["second-file type documentation".into()];
    let id = fixture.builder.generated_type(GeneratedType {
        name: "Earlier".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::RustSource(std::sync::Arc::new(origin)),
        source: records::source(),
    });
    let other = JavaTypeDeclaration {
        declared: Some(id),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: records::name("Earlier"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![],
    };
    records::add_file(
        &mut fixture.builder,
        "Earlier",
        vec![GeneratedSymbolId::Type(id)],
        other,
    );
    let rendered = records::certify(fixture.finish());
    assert_eq!(rendered.files().len(), 2);
    for file in rendered.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("Java text")
        };
        let is_facade = file.path().ends_with("/Generated.java");
        for module_doc in [
            "crate docs first",
            "private module documentation",
            "alias-only module documentation",
        ] {
            assert_eq!(text.matches(module_doc).count(), usize::from(is_facade));
        }
        assert_eq!(
            text.matches("second-file type documentation").count(),
            usize::from(!is_facade)
        );
    }
}

#[test]
fn multiple_presentation_owners_reject_before_linking() {
    let mut fixture = fixture::fixture(records::Facade::EntryPoint);
    let id = fixture.builder.generated_type(GeneratedType {
        name: "Generated".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
        source: records::source(),
    });
    let mut duplicate = fixture.facade.clone();
    duplicate.declared = Some(id);
    duplicate.members.clear();
    records::add_file(
        &mut fixture.builder,
        "Generated",
        vec![GeneratedSymbolId::Type(id)],
        duplicate,
    );
    let errors = JavaDialect.verify_package(&fixture.finish());
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("exactly one emitted PackageEntryPoint")),
        "{errors:?}"
    );
}
