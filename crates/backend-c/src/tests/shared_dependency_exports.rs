//! A selected function imports its owner's entire public header/object surface.
use super::*;

fn api(crate_id: u64, names: &[&str]) -> CDependencyApi {
    let input = fixture::named(crate_id, names, &[], &vec![None; names.len()]);
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, fixture::linked(&input)).unwrap(),
    )
    .unwrap()
}

fn selected(api: &CDependencyApi) -> CDependencyFunction {
    api.functions().next().unwrap().clone()
}

fn reject(names: &[&str], dependencies: &[CDependencyFunction], calls: &[Option<usize>]) {
    let input = fixture::named(90, names, dependencies, calls);
    let ast = project_c_package(input.registry, input.files).unwrap();
    let checked = verify_unresolved_package(&CDialect, ast).unwrap();
    assert!(TargetLinker::new(CDialect).link_ast(&checked).is_err());
}

#[test]
fn owned_binding_cannot_collide_with_an_unselected_dependency_export() {
    let owner = api(70, &["poly_selected", "poly_other_export"]);
    for call in [None, Some(0)] {
        reject(&["poly_other_export"], &[selected(&owner)], &[call]);
        reject(&["other_export"], &[selected(&owner)], &[call]);
    }
}

#[test]
fn reserving_full_exports_does_not_invent_imported_calls_or_definitions() {
    let owner = api(70, &["poly_selected", "poly_other_export"]);
    let input = fixture::named(90, &["poly_consumer"], &[selected(&owner)], &[Some(0)]);
    let linked = fixture::linked(&input);
    assert!(verify_linked_package(&linked).is_ok());
    let imports: Vec<_> = linked
        .files()
        .iter()
        .flat_map(|file| file.imports())
        .filter(|import| matches!(import.kind(), CImportKind::Dependency(_)))
        .collect();
    assert_eq!(imports.len(), 1);
    assert_eq!(
        linked
            .files()
            .iter()
            .flat_map(|file| file.items())
            .flat_map(|unit| unit.unit.data.bindings.imports.values())
            .count(),
        1
    );
    assert_eq!(imports[0].binding().as_str(), "poly_selected");
    assert_eq!(linked.files().len(), 2);
}

#[test]
fn selected_and_unselected_exports_of_different_owners_cannot_collide() {
    let first = api(70, &["poly_selected", "poly_other_export"]);
    let second = api(80, &["poly_other_export"]);
    reject(
        &["poly_consumer_a", "poly_consumer_b"],
        &[selected(&first), selected(&second)],
        &[Some(0), Some(1)],
    );
}

#[test]
fn unselected_exports_of_different_owners_cannot_collide() {
    let first = api(70, &["poly_selected", "poly_other_export"]);
    let second = api(80, &["poly_second_selected", "poly_other_export"]);
    reject(
        &["poly_consumer_a", "poly_consumer_b"],
        &[selected(&first), selected(&second)],
        &[Some(0), Some(1)],
    );
}
