//! Private paired grammar/platform proof, before public package admission.
use super::{
    package_fixture::{Fixture, fixture},
    platform, profile,
};
use crate::ast::*;

fn replace(fixture: &mut Fixture, index: usize, items: Vec<CFileItem>) {
    fixture.files[index] = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[index].identity().clone(),
    )
    .unwrap()
    .source_file(items)
    .unwrap();
}

#[test]
fn paired_traversal_is_header_first_and_platform_obligations_are_header_owned() {
    let fixture = fixture();
    let mut files = fixture.files.clone();
    files.reverse();
    let mut owners = vec![];
    profile::walk_package(&files, |source, node, _| {
        if let profile::Node::Item(_) = node {
            owners.push(source.identity().clone());
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(owners[0], *fixture.public.file());
    let installed = platform::install_package(&fixture.registry, files).unwrap();
    platform::verify_package(&installed).unwrap();
    for source in &installed {
        let assertions = source
            .items()
            .iter()
            .filter(|item| matches!(item, CFileItem::StaticAssert(_)))
            .count();
        assert_eq!(
            assertions,
            if source.identity() == fixture.public.file() {
                10
            } else {
                0
            }
        );
    }
    assert_eq!(
        installed,
        platform::install_package(&fixture.registry, installed.clone()).unwrap()
    );
    fixture
        .registry
        .registrations()
        .check_context(&installed)
        .unwrap();
}

#[test]
fn paired_grammar_rejects_missing_duplicate_and_late_prototypes_and_widened_helpers() {
    for mutation in 0..4 {
        let mut fixture = fixture();
        match mutation {
            0 => replace(&mut fixture, 0, vec![]),
            1 => {
                let mut items = fixture.files[0].items().to_vec();
                items.push(items[0].clone());
                replace(&mut fixture, 0, items);
            }
            2 => {
                let mut items = fixture.files[1].items().to_vec();
                let prototype = items.remove(0);
                items.push(prototype);
                replace(&mut fixture, 1, items);
            }
            3 => {
                let builder = CDeclarations::new(
                    fixture.registry.registrations(),
                    fixture.helper.file().clone(),
                )
                .unwrap();
                let prototype = builder
                    .function_prototype(fixture.helper.clone(), CLinkage::External)
                    .unwrap();
                let mut items = fixture.files[1].items().to_vec();
                items[0] = CFileItem::Declaration(prototype);
                replace(&mut fixture, 1, items);
            }
            _ => unreachable!(),
        }
        assert!(
            profile::check_package(&fixture.files).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn platform_reconstruction_rejects_missing_or_relocated_assertions() {
    let mut fixture = fixture();
    fixture.files = platform::install_package(&fixture.registry, fixture.files.clone()).unwrap();
    let mut header_items = fixture.files[0].items().to_vec();
    let CFileItem::StaticAssert(assertion) = header_items.remove(0) else {
        panic!("installed assertion")
    };
    replace(&mut fixture, 0, header_items);
    assert!(platform::verify_package(&fixture.files).is_err());
    assert!(platform::install_package(&fixture.registry, fixture.files.clone()).is_err());
    let builder = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.helper.file().clone(),
    )
    .unwrap();
    let moved = builder
        .static_assert(
            assertion.condition().clone(),
            assertion.diagnostic().clone(),
        )
        .unwrap();
    let mut source_items = fixture.files[1].items().to_vec();
    source_items.insert(0, CFileItem::StaticAssert(moved));
    replace(&mut fixture, 1, source_items);
    assert!(profile::check_package(&fixture.files).is_err());
}
