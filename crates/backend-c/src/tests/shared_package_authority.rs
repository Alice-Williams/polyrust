//! Whole authority and per-file data must remain an exact coherent projection.
use super::*;
use crate::{
    ast::CScalarType,
    dialect::shared::{CDialect, project_c_package, tests::fixture},
};
use portable_codegen::{TargetAstBuilder, TargetAstPackage, TargetFile, verify_unresolved_package};

fn change(
    package: &TargetAstPackage<CDialect>,
    mutate: impl FnOnce(&mut CProjectedUnit),
) -> TargetAstPackage<CDialect> {
    let mut builder = TargetAstBuilder::new(CDialect);
    for ty in package.generated_types() {
        builder.generated_type(ty.clone());
    }
    for callable in package.callables() {
        builder.callable(callable.clone());
    }
    for value in package.values() {
        builder.value(value.clone());
    }
    let file = package.files().next().unwrap();
    let mut units = file.items().to_vec();
    mutate(&mut units[0]);
    builder.file(TargetFile::new(
        file.path().clone(),
        file.role(),
        file.module().clone(),
        *file.placement(),
        units,
        file.source_kind().clone(),
        file.source().clone(),
    ));
    for group in package.groups() {
        builder.group(group.clone());
    }
    builder.build()
}

#[test]
fn one_immutable_source_is_shared_between_authority_and_file_projection() {
    let (registry, source) = fixture(CScalarType::I32);
    let package = project_c_package(registry, vec![source]).unwrap();
    let unit = &package.files().next().unwrap().items()[0];
    assert_eq!(unit.projection.sources.len(), 1);
    assert!(Arc::ptr_eq(&unit.projection.sources[0], &unit.data.source));
    assert_eq!(unit.projection.bindings, unit.data.bindings);
    verify_unresolved_package(&CDialect, package).unwrap();
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    MissingSource,
    ExtraSource,
    ForeignSource,
    ForeignFileData,
    ForeignRegistry,
    MissingPackageBinding,
    MissingFileBinding,
    MissingPrimaryDeclaration,
}

#[test]
fn partial_or_foreign_authorities_and_file_snapshots_do_not_verify() {
    let (registry, source) = fixture(CScalarType::I32);
    let package = project_c_package(registry, vec![source]).unwrap();
    let (foreign_registry, foreign_source) = fixture(CScalarType::I32);
    let foreign_package = project_c_package(foreign_registry, vec![foreign_source]).unwrap();
    let foreign = &foreign_package.files().next().unwrap().items()[0];
    for mutation in [
        Mutation::MissingSource,
        Mutation::ExtraSource,
        Mutation::ForeignSource,
        Mutation::ForeignFileData,
        Mutation::ForeignRegistry,
        Mutation::MissingPackageBinding,
        Mutation::MissingFileBinding,
        Mutation::MissingPrimaryDeclaration,
    ] {
        let changed = change(&package, |unit| match mutation {
            Mutation::MissingSource => Arc::make_mut(&mut unit.projection).sources.clear(),
            Mutation::ExtraSource => {
                let duplicate = unit.data.source.clone();
                Arc::make_mut(&mut unit.projection).sources.push(duplicate);
            }
            Mutation::ForeignSource => {
                Arc::make_mut(&mut unit.projection).sources[0] = foreign.data.source.clone()
            }
            Mutation::ForeignFileData => unit.data = foreign.data.clone(),
            Mutation::ForeignRegistry => {
                Arc::make_mut(&mut unit.projection).registry = foreign.projection.registry.clone()
            }
            Mutation::MissingPackageBinding => Arc::make_mut(&mut unit.projection)
                .bindings
                .functions
                .clear(),
            Mutation::MissingFileBinding => {
                Arc::make_mut(&mut unit.data).bindings.functions.clear()
            }
            Mutation::MissingPrimaryDeclaration => {
                Arc::make_mut(&mut unit.data).declarations.clear()
            }
        });
        assert!(
            verify_unresolved_package(&CDialect, changed).is_err(),
            "{mutation:?}"
        );
    }
}
