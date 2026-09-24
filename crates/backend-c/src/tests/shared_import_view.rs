//! Separate exact consumer imports and local definitions across multiple units.
use super::*;
use crate::{
    ast::CScalarType,
    dialect::shared::{c_defined_functions, dependency_fixture},
};
use portable_codegen::certify_resolved_package;

#[test]
fn view_reports_each_registration_once_and_retains_its_exact_certificate() {
    let owner = dependency_fixture::api(51, &[CScalarType::I32, CScalarType::Bool]);
    let dependencies: Vec<_> = owner.functions().cloned().collect();
    for call in [None, Some(0)] {
        let fixture = dependency_fixture::fixture(52, &[CScalarType::I32], &dependencies, &[call]);
        let package =
            certify_resolved_package(&CDialect, dependency_fixture::linked(&fixture)).unwrap();
        let imports: Vec<_> = c_imported_functions(&package).collect();
        let output = portable_codegen::render_certified_package(
            &crate::dialect::CStructuralRenderer,
            &package,
        )
        .unwrap();
        let bytes: usize = output
            .files()
            .iter()
            .map(|file| match file.contents() {
                portable_codegen::OutputContents::Text(text) => text.len(),
                _ => panic!("expected C text"),
            })
            .sum();
        assert!(crate::dialect::c_output_byte_bound(&package).unwrap() >= bytes as u64);
        assert_eq!(imports.len(), 2);
        for (index, imported) in imports.iter().enumerate() {
            assert_eq!(imported.function(), &fixture.imported[index]);
            assert_eq!(imported.dependency(), &dependencies[index]);
            assert_ne!(imported.function(), imported.dependency().function());
            assert_eq!(
                imported.function().signature(),
                imported.dependency().signature()
            );
            assert_eq!(
                imported
                    .dependency()
                    .package_identity()
                    .source_root()
                    .unwrap(),
                owner.source_root().unwrap()
            );
        }
        let definitions: Vec<_> = c_defined_functions(&package)
            .map(|item| item.function())
            .collect();
        assert_eq!(definitions, fixture.functions.iter().collect::<Vec<_>>());
        assert!(
            imports
                .iter()
                .all(|item| !definitions.contains(&item.function()))
        );
    }
    assert_eq!(c_imported_functions(owner.package()).count(), 0);
}
