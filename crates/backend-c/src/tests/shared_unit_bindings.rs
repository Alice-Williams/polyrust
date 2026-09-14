//! Primary declarations and body-local names do not follow the same file rule.
use super::*;
use crate::dialect::{
    file_dependencies,
    shared::{CDialect, package_fixture::fixture, registration},
};
use portable_codegen::TargetAstBuilder;

#[test]
fn header_declares_the_public_identity_but_implementation_only_refers_to_it() {
    let fixture = fixture();
    let registry = fixture.registry.registrations();
    registry.check_context(&fixture.files).unwrap();
    crate::dialect::shared::profile::check_package(&fixture.files).unwrap();
    let mut builder = TargetAstBuilder::new(CDialect);
    let bindings = registration::register(&mut builder, registry, &fixture.files).unwrap();
    let dependencies = file_dependencies(&fixture.registry, &fixture.files).unwrap();
    let header = dependencies
        .iter()
        .find(|file| file.file() == fixture.public.file())
        .unwrap();
    let source = dependencies
        .iter()
        .find(|file| file.file() == fixture.helper.file())
        .unwrap();
    let (header_bindings, header_declarations) =
        project(&bindings, &fixture.files, header).unwrap();
    let (source_bindings, source_declarations) =
        project(&bindings, &fixture.files, source).unwrap();
    let public = Symbol::Callable(bindings.functions[&fixture.public]);
    let private = Symbol::Callable(bindings.functions[&fixture.helper]);
    assert_eq!(header_declarations, [public]);
    assert_eq!(header_bindings.symbols(), [public]);
    assert!(!source_declarations.contains(&public));
    assert!(source_declarations.contains(&private));
    assert!(source_bindings.symbols().contains(&public));
    assert!(source_bindings.symbols().contains(&private));
    // The public parameter and its local are private implementation bindings,
    // despite the function's authoritative declaration belonging to the header.
    assert!(header_bindings.values.is_empty());
    assert_eq!(source_bindings.values.len(), 3);
    for id in source_bindings.values.values() {
        assert!(source_declarations.contains(&Symbol::Value(*id)));
    }
    assert_eq!(source_declarations.len(), 4);
}

#[test]
fn missing_body_cannot_silently_place_parameters_in_the_primary_header() {
    let fixture = fixture();
    let mut builder = TargetAstBuilder::new(CDialect);
    let bindings = registration::register(
        &mut builder,
        fixture.registry.registrations(),
        &fixture.files,
    )
    .unwrap();
    let dependencies = file_dependencies(&fixture.registry, &fixture.files).unwrap();
    assert!(project(&bindings, &fixture.files[..1], &dependencies[0]).is_err());
}
