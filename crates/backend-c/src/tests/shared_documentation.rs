//! Typed owner inventory cannot be detached, changed or reordered after lowering.
use super::*;
use crate::dialect::shared::{CDialect, CStructuralRenderer, project_c_package};
use portable_codegen::*;
use std::sync::Arc;

#[path = "shared_documentation_fixture.rs"]
mod fixture;
#[path = "shared_documentation_resources.rs"]
mod resource_tests;

fn package(metadata: RustSourceOrigin) -> TargetAstPackage<CDialect> {
    let (registry, source) = fixture::source(vec![metadata]);
    project_c_package(registry, vec![source]).unwrap()
}

#[test]
fn documentation_is_normalized_once_and_attached_to_primary_owners() {
    let mut other = fixture::metadata();
    other.declaration.definition_path_hash = 4;
    other.documentation = vec!["other function".into()];
    let (registry, source) = fixture::source(vec![fixture::metadata(), other]);
    let package = project_c_package(registry, vec![source]).unwrap();
    let unit = &package.files().next().unwrap().items()[0];
    assert_eq!(
        unit.data
            .documentation
            .modules()
            .map(CComment::text)
            .collect::<Vec<_>>(),
        ["root", "child"]
    );
    assert_eq!(unit.data.documentation.all().count(), 6);
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let rendered = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    let OutputContents::Text(text) = rendered.files()[0].contents() else {
        panic!("source")
    };
    for value in [
        "root",
        "child",
        "function one",
        "function two * /",
        "other function",
    ] {
        assert_eq!(text.matches(value).count(), 1, "duplicate/missing {value}");
    }
    assert!(text.contains(
        "/* function one */\n/*  */\n/* function two * / */\nint32_t poly_document0(void);"
    ));
}

fn tamper(
    package: &TargetAstPackage<CDialect>,
    mutation: impl FnOnce(&mut CDocumentation),
) -> TargetAstPackage<CDialect> {
    let mut builder = TargetAstBuilder::new(CDialect);
    for ty in package.generated_types() {
        builder.generated_type(ty.clone());
    }
    for function in package.callables() {
        builder.callable(function.clone());
    }
    for value in package.values() {
        builder.value(value.clone());
    }
    let file = package.files().next().unwrap();
    let mut items = file.items().to_vec();
    mutation(&mut Arc::make_mut(&mut items[0].data).documentation);
    builder.file(TargetFile::new(
        file.path().clone(),
        file.role(),
        file.module().clone(),
        *file.placement(),
        items,
        file.source_kind().clone(),
        file.source().clone(),
    ));
    for group in package.groups() {
        builder.group(group.clone());
    }
    builder.build()
}

#[test]
fn documentation_projection_rejects_lost_text_changed_text_and_module_order() {
    let package = package(fixture::metadata());
    for mutation in [0, 1, 2, 3] {
        let changed = tamper(&package, |docs| match mutation {
            0 => docs.attachments.clear(),
            1 => *docs.attachments.values_mut().next().unwrap() = vec![CComment::new("forged")],
            2 => docs.module_order.clear(),
            3 => docs.module_order.reverse(),
            _ => unreachable!(),
        });
        assert!(verify_unresolved_package(&CDialect, changed).is_err());
    }
}

#[test]
fn documentation_module_ancestry_is_exact_and_consistent_across_owners() {
    for mutation in [0, 1, 2, 3, 4] {
        let mut metadata = fixture::metadata();
        match mutation {
            0 => metadata.module_ancestors = [].into(),
            1 => fixture::module(&mut metadata, 1).parent = None,
            2 => fixture::module(&mut metadata, 1).declaration.crate_id += 1,
            3 => {
                let mut ancestors = metadata.module_ancestors.to_vec();
                ancestors.push(ancestors[1].clone());
                metadata.module_ancestors = ancestors.into();
            }
            4 => metadata.module.definition_path_hash += 1,
            _ => unreachable!(),
        }
        let (registry, source) = fixture::source(vec![metadata]);
        assert!(project_c_package(registry, vec![source]).is_err());
    }
    let mut other = fixture::metadata();
    other.declaration.definition_path_hash = 4;
    fixture::module(&mut other, 1)
        .documentation
        .push("conflicting".into());
    let (registry, source) = fixture::source(vec![fixture::metadata(), other]);
    let errors = project_c_package(registry, vec![source]).unwrap_err();
    assert!(format!("{errors:?}").contains("conflicting documentation"));
}

#[test]
fn documentation_source_identity_cannot_select_two_distinct_target_owners() {
    for metadata in [fixture::metadata(), fixture::bare()] {
        let (registry, source) = fixture::source(vec![metadata.clone(), metadata]);
        let errors = project_c_package(registry, vec![source]).unwrap_err();
        assert!(format!("{errors:?}").contains("conflicting documentation owners"));
    }
}

#[test]
fn documentation_one_crate_cannot_have_disconnected_module_roots() {
    let first = fixture::metadata();
    let mut second = first.clone();
    second.declaration.definition_path_hash = 100;
    second.module.definition_path_hash = 101;
    fixture::module(&mut second, 0)
        .declaration
        .definition_path_hash = 102;
    fixture::module(&mut second, 1).declaration = second.module;
    fixture::module(&mut second, 1).parent = Some(second.module_ancestors[0].declaration);
    let (registry, source) = fixture::source(vec![first, second]);
    let errors = project_c_package(registry, vec![source]).unwrap_err();
    assert!(format!("{errors:?}").contains("export root disagrees with documentation ancestry"));
}

#[test]
fn documentation_module_and_declaration_cannot_share_source_identity() {
    let mut metadata = fixture::metadata();
    metadata.declaration = metadata.module;
    let (registry, source) = fixture::source(vec![metadata]);
    assert!(project_c_package(registry, vec![source]).is_err());
}

#[test]
fn documentation_primary_prototype_is_unique_even_when_attributes_are_empty() {
    for metadata in [fixture::metadata(), fixture::bare()] {
        let (registry, source) = fixture::source(vec![metadata]);
        let mut items = source.items().to_vec();
        items.insert(0, items[0].clone());
        let source =
            crate::ast::CDeclarations::new(registry.registrations(), source.identity().clone())
                .unwrap()
                .source_file(items)
                .unwrap();
        let errors = project_c_package(registry, vec![source]).unwrap_err();
        assert!(format!("{errors:?}").contains("exactly one primary prototype"));
    }
}
