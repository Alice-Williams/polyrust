//! Canonical registry projection and independent whole-graph reconstruction.

use super::{
    CDialect, CFileGrammar, CStdType, diagnostic,
    package::{CProjectedUnit, CProjection, CUnitData},
    profile, violation,
};
use crate::ast::{CFileRole, CFrozenRegistry, CSourceFile};
use crate::dialect::{CHeader, file_dependencies};
use portable_codegen::{
    AstViolation, FileGroupRole, SourceRole, TargetAstBuilder, TargetAstPackage, TargetFile,
    TargetFileGroup, TargetFileMember,
};
use portable_diagnostics::{Diagnostic, SourceRef};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Produces the shared unresolved C graph, NOT a render-ready package.
pub fn project_c_package(
    registry: CFrozenRegistry,
    sources: Vec<CSourceFile>,
) -> Result<TargetAstPackage<CDialect>, Vec<Diagnostic>> {
    let sources = super::platform::install_package(&registry, sources).map_err(diagnostic)?;
    build(registry, sources).map_err(diagnostic)
}

fn build(
    registry: CFrozenRegistry,
    mut sources: Vec<CSourceFile>,
) -> Result<TargetAstPackage<CDialect>, String> {
    profile::check_registered_package(registry.registrations(), &sources)?;
    super::platform::verify_package(&sources)?;
    // Shared storage is path ordered; grammar traversal separately visits the
    // public header before definitions. Neither depends on caller ordering.
    sources.sort_by(|left, right| left.identity().key().path.cmp(&right.identity().key().path));
    let files = sources.as_slice();
    let registrations = registry.registrations();
    registrations
        .check_context(files)
        .map_err(|e| e.to_string())?;
    registrations
        .check_constants_and_layout(files)
        .map_err(|e| e.to_string())?;
    registrations
        .check_sequencing_and_control(files)
        .map_err(|e| e.to_string())?;
    registrations
        .check_numeric_flow(files)
        .map_err(|e| e.to_string())?;
    registrations
        .check_index_extents(files)
        .map_err(|e| e.to_string())?;
    registrations
        .check_storage_paths(files)
        .map_err(|e| e.to_string())?;
    let dependencies: BTreeMap<_, _> = file_dependencies(&registry, files)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|dependencies| (dependencies.file().clone(), dependencies))
        .collect();
    let mut builder = TargetAstBuilder::new(CDialect);
    let bindings = super::registration::register(&mut builder, registrations, files)?;
    let mut documentation = super::documentation::lower_package(files)?;
    let header_grammar = match profile::ordered_sources(files)?.as_slice() {
        [header, implementation] => Some(CFileGrammar::Header(
            super::CGeneratedHeader::resolve(
                registrations,
                implementation.identity(),
                header.identity(),
            )
            .map_err(|error| error.message)?
            .guard()
            .clone(),
        )),
        [_] => None,
        _ => return Err("C package has no checked file layout".into()),
    };
    let mut data = Vec::new();
    for source in files {
        let dependencies = dependencies
            .get(source.identity())
            .ok_or("C file has no exact dependency inventory")?;
        let (file_bindings, declarations) =
            super::unit_bindings::project(&bindings, files, dependencies)?;
        let mut standards = BTreeSet::new();
        for header in dependencies.headers() {
            standards.insert(match header {
                CHeader::Stdint => CStdType::I32,
                CHeader::Stddef => CStdType::Size,
                _ => return Err("header is outside the first C profile".into()),
            });
        }
        data.push(CUnitData {
            source: Arc::new(source.clone()),
            bindings: file_bindings,
            declarations,
            standards,
            documentation: documentation
                .remove(source.identity())
                .ok_or("C file has no documentation projection")?,
        });
    }
    let projection = Arc::new(CProjection {
        registry,
        sources: data.iter().map(|data| data.source.clone()).collect(),
        bindings,
    });
    let mut groups = Vec::new();
    for data in data {
        let file = data.source.identity().clone();
        let (role, group, grammar) = match file.key().role {
            CFileRole::GeneratedPublicHeader => (
                SourceRole::PublicApi,
                FileGroupRole::PublicApi,
                header_grammar
                    .clone()
                    .ok_or("C header has no checked guard")?,
            ),
            CFileRole::GeneratedSource => (
                SourceRole::Implementation,
                FileGroupRole::Implementation,
                CFileGrammar::TranslationUnit,
            ),
            CFileRole::TestSource => (
                SourceRole::NativeTest,
                FileGroupRole::NativeTests,
                CFileGrammar::TranslationUnit,
            ),
            _ => return Err("unsupported C package file role".into()),
        };
        let unit = CProjectedUnit {
            projection: projection.clone(),
            data: Arc::new(data),
        };
        let location = SourceRef::logical(["c", file.key().path.as_str()]);
        let id = builder.file(TargetFile::new(
            file.key().path.clone(),
            role,
            file.clone(),
            file.key().role,
            vec![unit],
            grammar,
            location.clone(),
        ));
        groups.push(TargetFileGroup::new(
            group,
            vec![TargetFileMember::Source(id)],
            location,
        ));
    }
    groups.sort_by_key(TargetFileGroup::role);
    for group in groups {
        builder.group(group);
    }
    Ok(builder.build())
}

pub(super) fn verify(package: &TargetAstPackage<CDialect>) -> Vec<AstViolation> {
    let Some(unit) = package.files().next().and_then(|file| file.items().first()) else {
        return vec![violation(
            "C shared package has no authoritative registry projection",
        )];
    };
    let projection = &unit.projection;
    match build(
        projection.registry.clone(),
        projection
            .sources
            .iter()
            .map(|source| source.as_ref().clone())
            .collect(),
    ) {
        Ok(expected) if &expected == package => vec![],
        Ok(_) => vec![violation(
            "C shared graph differs from its exact bidirectional registry projection",
        )],
        Err(message) => vec![violation(message)],
    }
}
