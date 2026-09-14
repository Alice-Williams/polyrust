//! Whole-package reconstruction rejects plausible but mismatched file snapshots.
use super::{
    CDialect, CFileGrammar, CGeneratedHeader, CVisibility, package_fixture, project_c_package,
};
use portable_codegen::{TargetAstBuilder, TargetAstPackage, TargetFile, verify_unresolved_package};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    None,
    MissingHeader,
    MissingSource,
    ExtraHeader,
    MissingPrimary,
    ForeignGuard,
    MissingGuard,
    GuardOnSource,
    SwappedData,
    ForeignAuthority,
    WidenedPrivateFunction,
    BodyLocalInHeader,
}

fn changed(package: &TargetAstPackage<CDialect>, mutation: Mutation) -> TargetAstPackage<CDialect> {
    let mut builder = TargetAstBuilder::new(CDialect);
    for ty in package.generated_types() {
        builder.generated_type(ty.clone());
    }
    for callable in package.callables() {
        let mut callable = callable.clone();
        if matches!(mutation, Mutation::WidenedPrivateFunction) {
            callable.visibility = CVisibility::Exported;
        }
        builder.callable(callable);
    }
    for value in package.values() {
        builder.value(value.clone());
    }
    let foreign = package_fixture::fixture();
    let foreign_guard = CGeneratedHeader::resolve(
        foreign.registry.registrations(),
        foreign.helper.file(),
        foreign.public.file(),
    )
    .unwrap()
    .guard()
    .clone();
    let foreign = project_c_package(foreign.registry, foreign.files).unwrap();
    let source = package
        .files()
        .find(|file| matches!(file.source_kind(), CFileGrammar::TranslationUnit))
        .unwrap();
    let header = package
        .files()
        .find(|file| matches!(file.source_kind(), CFileGrammar::Header(_)))
        .unwrap();
    for file in package.files() {
        let is_header = matches!(file.source_kind(), CFileGrammar::Header(_));
        if matches!(mutation, Mutation::MissingHeader) && is_header
            || matches!(mutation, Mutation::MissingSource) && !is_header
        {
            continue;
        }
        let mut grammar = file.source_kind().clone();
        let mut units = file.items().to_vec();
        match mutation {
            Mutation::ForeignGuard if is_header => {
                grammar = CFileGrammar::Header(foreign_guard.clone())
            }
            Mutation::MissingGuard if is_header => grammar = CFileGrammar::TranslationUnit,
            Mutation::GuardOnSource if !is_header => grammar = header.source_kind().clone(),
            Mutation::MissingPrimary if is_header => {
                Arc::make_mut(&mut units[0].data).declarations.clear()
            }
            Mutation::SwappedData if is_header => units[0].data = source.items()[0].data.clone(),
            Mutation::ForeignAuthority => {
                units[0].projection = foreign.files().next().unwrap().items()[0]
                    .projection
                    .clone()
            }
            Mutation::BodyLocalInHeader if is_header => {
                let local = source.items()[0]
                    .data
                    .declarations
                    .iter()
                    .find(|symbol| matches!(symbol, portable_codegen::GeneratedSymbolId::Value(_)))
                    .unwrap();
                Arc::make_mut(&mut units[0].data).declarations.push(*local);
            }
            _ => {}
        }
        let rebuilt = TargetFile::new(
            file.path().clone(),
            file.role(),
            file.module().clone(),
            *file.placement(),
            units,
            grammar,
            file.source().clone(),
        );
        builder.file(rebuilt.clone());
        if matches!(mutation, Mutation::ExtraHeader) && is_header {
            builder.file(rebuilt);
        }
    }
    for group in package.groups() {
        builder.group(group.clone());
    }
    builder.build()
}

#[test]
fn no_paired_snapshot_guard_or_visibility_mutation_verifies() {
    let fixture = package_fixture::fixture();
    let package = project_c_package(fixture.registry, fixture.files).unwrap();
    verify_unresolved_package(&CDialect, changed(&package, Mutation::None)).unwrap();
    for mutation in [
        Mutation::MissingHeader,
        Mutation::MissingSource,
        Mutation::ExtraHeader,
        Mutation::MissingPrimary,
        Mutation::ForeignGuard,
        Mutation::MissingGuard,
        Mutation::GuardOnSource,
        Mutation::SwappedData,
        Mutation::ForeignAuthority,
        Mutation::WidenedPrivateFunction,
        Mutation::BodyLocalInHeader,
    ] {
        assert!(
            verify_unresolved_package(&CDialect, changed(&package, mutation)).is_err(),
            "{mutation:?}"
        );
    }
}
