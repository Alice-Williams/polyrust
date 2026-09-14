//! Actual independently certified scalar packages flow through shared linking.
use super::*;
use crate::{ast::*, dialect::*};
use portable_codegen::*;
use std::collections::BTreeSet;

use crate::dialect::shared::dependency_fixture as fixture;

#[path = "shared_dependency_collisions.rs"]
mod collisions;

#[path = "shared_dependency_exports.rs"]
mod export_tests;

fn dependencies() -> Vec<CDependencyFunction> {
    let first = fixture::api(70, &[CScalarType::I32, CScalarType::Bool]);
    let second = fixture::api(80, &[CScalarType::I32]);
    first
        .functions()
        .chain(second.functions())
        .cloned()
        .collect()
}

#[test]
fn independent_public_functions_keep_fixed_names_and_one_include_per_header() {
    let dependencies = dependencies();
    let fixture = fixture::fixture(
        90,
        &[CScalarType::I32, CScalarType::Bool, CScalarType::I32],
        &dependencies,
        &[Some(0), Some(1), Some(2)],
    );
    let linked = fixture::linked(&fixture);
    assert_eq!(linked, fixture::linked(&fixture));
    assert!(verify_linked_package(&linked).is_ok());
    assert!(linked.dependencies().is_empty()); // No invented package-manager requirements.
    assert_eq!(linked.files().len(), 2);
    let header = linked
        .files()
        .iter()
        .find(|file| file.module().key().role == CFileRole::GeneratedPublicHeader)
        .unwrap();
    let source = linked
        .files()
        .iter()
        .find(|file| file.module().key().role == CFileRole::GeneratedSource)
        .unwrap();
    assert!(
        !header
            .imports()
            .iter()
            .any(|import| matches!(import.kind(), CImportKind::Dependency(_)))
    );
    let imports: Vec<_> = source
        .imports()
        .iter()
        .filter(|import| matches!(import.kind(), CImportKind::Dependency(_)))
        .collect();
    assert_eq!(imports.len(), 3);
    let code = super::super::spelling::file(source);
    for path in ["polyrust_dep_70.h", "polyrust_dep_80.h"] {
        assert_eq!(
            code.matches(&format!("#include \"{path}\"")).count(),
            1,
            "{code}"
        );
    }
    let unit = &source.items()[0];
    assert_eq!(unit.unit.data.bindings.imports.len(), 3);
    let owned: BTreeSet<_> = unit
        .unit
        .data
        .source
        .items()
        .iter()
        .filter_map(|item| {
            if let CFileItem::Definition(value) = item
                && let CDefinitionKind::Function { function, .. } = value.kind()
            {
                Some(function.clone())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(owned, fixture.functions.iter().cloned().collect());
    for (index, dependency) in dependencies.iter().enumerate() {
        let consumer = &fixture.imported[index];
        assert_ne!(consumer, dependency.function());
        let imported = unit.unit.data.bindings.imports.get(consumer).unwrap();
        assert_eq!(imported.function(), consumer);
        assert_eq!(imported.dependency(), dependency);
        assert_eq!(
            unit.spelling.functions.get(consumer),
            Some(dependency.symbol())
        );
        assert!(!owned.contains(consumer));
        assert_eq!(
            code.matches(&format!("{}(", dependency.symbol().as_str()))
                .count(),
            1,
            "{code}"
        );
        assert!(
            imports
                .iter()
                .any(|import| import.binding() == dependency.symbol()
                    && import.original_binding() == dependency.symbol()
                    && import.kind() == &CImportKind::Dependency(dependency.package_identity()))
        );
    }
    assert!(certify_resolved_package(&CDialect, linked).is_ok());
}

#[test]
fn unused_certified_dependencies_do_not_create_includes_or_local_declarations() {
    let dependencies = dependencies();
    let fixture = fixture::fixture(90, &[CScalarType::I32], &dependencies, &[None]);
    let ast = project_c_package(fixture.registry.clone(), fixture.files.clone()).unwrap();
    let catalogue = CDialect.package_symbol_catalogue(&ast).unwrap();
    assert_eq!(catalogue.dependency_callables.len(), 3);
    assert!(catalogue.verify(&CDialect).is_ok());
    let linked = fixture::linked(&fixture);
    for file in linked.files() {
        assert!(
            !file
                .imports()
                .iter()
                .any(|import| matches!(import.kind(), CImportKind::Dependency(_)))
        );
        assert!(file.items()[0].unit.data.bindings.imports.is_empty());
    }
    // All registered certificates are checked, but unused imports add no calls.
    assert!(certify_resolved_package(&CDialect, linked).is_ok());
}

#[test]
fn imported_metadata_and_resolved_name_mutations_fail_independent_checks() {
    let dependencies = dependencies();
    let fixture = fixture::fixture(90, &[CScalarType::I32], &dependencies, &[Some(0)]);
    let ast = project_c_package(fixture.registry.clone(), fixture.files.clone()).unwrap();
    let catalogue = CDialect.package_symbol_catalogue(&ast).unwrap();
    for fault in 0..4 {
        let mut changed = catalogue.clone();
        let spec = &mut changed.dependency_callables[0];
        match fault {
            0 => spec.name = CIdentifier::new("poly_invented").unwrap(),
            1 => spec.signature.parameters.clear(),
            2 => spec.owner = dependencies[2].package_identity(),
            3 => {
                spec.spelling = portable_codegen::DependencySpelling::FixedImport(
                    CImportKind::Dependency(dependencies[2].package_identity()),
                )
            }
            _ => unreachable!(),
        }
        assert!(changed.verify(&CDialect).is_err(), "fault {fault}");
    }
    let linked = fixture::linked(&fixture);
    let source = linked
        .files()
        .iter()
        .find(|file| file.module().key().role == CFileRole::GeneratedSource)
        .unwrap();
    let mut unit = source.items()[0].clone();
    let imported = unit
        .unit
        .data
        .bindings
        .imports
        .values()
        .next()
        .unwrap()
        .clone();
    let reference = unit
        .names
        .get_mut(&TargetSymbolRef::DependencyCallable(imported.clone()))
        .unwrap();
    let ResolvedReference::Imported { binding, .. } = reference else {
        panic!()
    };
    *binding = CIdentifier::new("poly_forged_alias").unwrap();
    unit.spelling
        .functions
        .insert(imported.function().clone(), binding.clone());
    assert!(!CDialect.verify_resolved_file_item(&unit).is_empty());
}
