//! Imported values use the same certified shared pipeline as callable imports.
use super::{
    CDialect, CImportKind, CImportedValue,
    bindings::CValueBinding,
    constant_consumer_fixture::{Usage, fixture, producer},
    owned_constant_fixture::Shape,
    owned_constant_tests::linked,
    project_c_package,
};
use crate::ast::*;
use portable_codegen::*;

#[test]
fn values_only_and_mixed_imports_keep_foreign_identity_and_one_native_header() {
    for shape in [Shape::ConstantsOnly, Shape::Mixed] {
        let owner = producer(shape);
        let values: Vec<_> = owner.constants().cloned().collect();
        let call = owner.functions().next().cloned();
        let input = fixture(90, &values, call.clone(), Usage::Read);
        let projected = project_c_package(input.registry.clone(), input.files.clone()).unwrap();
        let catalogue = CDialect.package_symbol_catalogue(&projected).unwrap();
        assert_eq!(catalogue.dependency_values.len(), values.len());
        assert_eq!(
            catalogue.dependency_callables.len(),
            usize::from(call.is_some())
        );
        assert!(catalogue.verify(&CDialect).is_ok());
        let package = linked(&input);
        let source = package
            .files()
            .iter()
            .find(|file| file.module().key().role == CFileRole::GeneratedSource)
            .unwrap();
        let unit = &source.items()[0];
        let text = super::spelling::file(source);
        assert_eq!(text.matches("#include \"polyrust_constants.h\"").count(), 1);
        assert_eq!(unit.unit.data.bindings.imported_values.len(), values.len());
        assert!(
            !unit
                .unit
                .data
                .bindings
                .values
                .keys()
                .any(|value| matches!(value, CValueBinding::Global(_)))
        );
        assert!(package.dependencies().is_empty());
        for (object, dependency) in input.objects.iter().zip(&values) {
            assert_ne!(object, dependency.object());
            assert_eq!(object.ty(), dependency.object().ty());
            assert_eq!(object.file(), dependency.object().file());
            let import = &unit.unit.data.bindings.imported_values[object];
            assert_eq!(import.dependency(), dependency);
            assert_eq!(import.object(), object);
            assert_eq!(
                unit.spelling.values[&CValueBinding::Global(object.clone())],
                *dependency.symbol()
            );
            assert!(
                input
                    .registry
                    .registrations()
                    .check_file(object.file())
                    .is_err()
            );
        }
        assert!(certify_resolved_package(&CDialect, package).is_ok());
    }
}

#[test]
fn unused_constant_registrations_keep_authority_without_generating_imports() {
    let owner = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    let input = fixture(90, &values, None, Usage::Unused);
    let projected = project_c_package(input.registry.clone(), input.files.clone()).unwrap();
    assert_eq!(
        CDialect
            .package_symbol_catalogue(&projected)
            .unwrap()
            .dependency_values
            .len(),
        values.len()
    );
    let package = linked(&input);
    for file in package.files() {
        assert!(
            file.items()[0]
                .unit
                .data
                .bindings
                .imported_values
                .is_empty()
        );
        assert!(
            !file
                .imports()
                .iter()
                .any(|value| matches!(value.kind(), CImportKind::Dependency(_)))
        );
    }
    assert!(certify_resolved_package(&CDialect, package).is_ok());
}

#[test]
fn exact_imported_numeric_facts_prove_extreme_subtractions_without_overflow() {
    let owner = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    let input = fixture(90, &values, None, Usage::Difference);
    assert!(
        input
            .registry
            .registrations()
            .check_storage_paths(&input.files)
            .is_ok()
    );
    // Arithmetic source admission is a later capability, not enabled incidentally.
    assert!(project_c_package(input.registry, input.files).is_err());
}

#[test]
fn constant_catalogue_and_resolved_binding_mutations_reject() {
    let owner = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    let input = fixture(90, &values, None, Usage::Read);
    let projected = project_c_package(input.registry.clone(), input.files.clone()).unwrap();
    let catalogue = CDialect.package_symbol_catalogue(&projected).unwrap();
    let other = producer(Shape::ConstantsOnly)
        .constants()
        .next()
        .unwrap()
        .package_identity();
    #[derive(Debug)]
    enum Fault {
        Name,
        Type,
        Owner,
        Import,
    }
    for fault in [Fault::Name, Fault::Type, Fault::Owner, Fault::Import] {
        let mut changed = catalogue.clone();
        let spec = &mut changed.dependency_values[0];
        match fault {
            Fault::Name => spec.name = CIdentifier::new("poly_invented").unwrap(),
            Fault::Type => {
                spec.ty = TargetTypeRef::Primitive(crate::dialect::CPrimitiveType::Scalar(
                    CScalarType::I64,
                ))
            }
            Fault::Owner => spec.owner = other.clone(),
            Fault::Import => {
                spec.spelling =
                    DependencySpelling::FixedImport(CImportKind::Dependency(other.clone()))
            }
        }
        assert!(changed.verify(&CDialect).is_err(), "{fault:?}");
    }
    let package = linked(&input);
    let source = package
        .files()
        .iter()
        .find(|file| file.module().key().role == CFileRole::GeneratedSource)
        .unwrap();
    let mut unit = source.items()[0].clone();
    let object = input.objects[0].clone();
    let import = CImportedValue::from_registry(input.registry.registrations(), &object).unwrap();
    unit.spelling.values.insert(
        CValueBinding::Global(object),
        CIdentifier::new("poly_false_alias").unwrap(),
    );
    assert!(!CDialect.verify_resolved_file_item(&unit).is_empty());
    unit.names.insert(
        TargetSymbolRef::DependencyValue(import),
        ResolvedReference::Local(CIdentifier::new("poly_false_alias").unwrap()),
    );
    assert!(!CDialect.verify_resolved_file_item(&unit).is_empty());
}
