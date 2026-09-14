//! Per-file bounds and one whole-definition call graph for a checked C package.
use super::{CResolvedUnit, profile};
use crate::ast::{CDefinitionKind, CFileItem, CFileRef};
use crate::ownership::layout::Layouts;
use std::collections::BTreeMap;

#[path = "call_paths.rs"]
mod call_paths;
#[path = "dependencies.rs"]
mod dependencies;
#[cfg(test)]
#[path = "../tests/shared_package_resource_integrity.rs"]
mod integrity_tests;
#[path = "hir_nodes.rs"]
mod nodes;
#[path = "hir_policy.rs"]
pub(super) mod policy;
#[path = "hir_syntax.rs"]
mod syntax;
#[path = "hir_totals.rs"]
mod totals;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Measurements {
    pub nodes: u64,
    pub depth: usize,
    pub max_parameters: usize,
    pub max_fields: usize,
    pub max_identifier_bytes: usize,
    pub comment_bytes: u64,
    pub diagnostic_bytes: u64,
    pub max_diagnostic_bytes: usize,
    pub automatic_bytes: u64,
    pub automatic_objects: u64,
    pub value_bytes: u64,
    pub source_bound: u64,
    pub frame_bound: u64,
    pub function_frames: BTreeMap<crate::ast::CFunctionRef, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PackageMeasurements {
    pub files: BTreeMap<CFileRef, Measurements>,
    pub total: Measurements,
}

fn add(total: &mut u64, value: u64) -> Result<(), String> {
    *total = total
        .checked_add(value)
        .ok_or("C resource measurement overflow")?;
    Ok(())
}

#[cfg(test)]
pub(super) fn measure(unit: &CResolvedUnit) -> Result<Measurements, String> {
    Ok(measure_units(&[unit])?.total)
}

fn measure_units(units: &[&CResolvedUnit]) -> Result<PackageMeasurements, String> {
    let authority = &units
        .first()
        .ok_or("C measurements require a package")?
        .unit
        .projection;
    let mut files = BTreeMap::new();
    for unit in units {
        if &unit.unit.projection != authority {
            return Err("C measurements require one complete package authority".into());
        }
        let data = &unit.unit.data;
        let mut measured = Measurements::default();
        for comment in data.documentation.all() {
            add(&mut measured.nodes, 1)?;
            add(&mut measured.comment_bytes, comment.text().len() as u64)?;
        }
        let names = &unit.spelling;
        measured.max_identifier_bytes = names
            .types
            .values()
            .chain(names.functions.values())
            .chain(names.values.values())
            .chain(names.standards.values())
            .map(|name| name.as_str().len())
            .max()
            .unwrap_or(0);
        if files
            .insert(data.source.identity().clone(), measured)
            .is_some()
        {
            return Err("duplicate C measurement file".into());
        }
    }
    if files.len() != authority.sources.len()
        || authority
            .sources
            .iter()
            .any(|source| !files.contains_key(source.identity()))
    {
        return Err("C measurements require the complete source inventory".into());
    }
    let sources: Vec<_> = authority
        .sources
        .iter()
        .map(|source| source.as_ref().clone())
        .collect();
    let mut layouts = Layouts::new(authority.registry.registrations());
    let mut calls = call_paths::Inventory::default();
    profile::walk_registered_package(
        authority.registry.registrations(),
        &sources,
        |source, node, depth| {
            let measured = files
                .get_mut(source.identity())
                .ok_or("missing C measurement file")?;
            let before = measured.clone();
            nodes::observe(measured, &mut layouts, node, depth)?;
            calls.observe(node, &before, measured)
        },
    )?;
    for measured in files.values_mut() {
        measured.source_bound = totals::source_bound(measured)?;
    }
    let mut total = totals::sum(files.values())?;
    (total.function_frames, total.frame_bound) =
        calls.finish(&total, authority.registry.registrations())?;
    // Frame identity follows the definition body, not its primary prototype.
    // A public header has syntax but never acquires an executable frame.
    for source in &sources {
        let measured = files
            .get_mut(source.identity())
            .ok_or("missing C measurement file")?;
        for item in source.items() {
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Function { function, .. } = definition.kind()
            {
                let bound = *total
                    .function_frames
                    .get(function)
                    .ok_or("missing C definition frame")?;
                measured.function_frames.insert(function.clone(), bound);
            }
        }
        if !measured.function_frames.is_empty() {
            measured.frame_bound = total.frame_bound;
        }
    }
    Ok(PackageMeasurements { files, total })
}

pub(super) fn measure_package(
    package: &portable_codegen::LinkedTargetPackage<super::CDialect>,
) -> Result<PackageMeasurements, String> {
    let units: Vec<_> = package
        .files()
        .iter()
        .flat_map(|file| file.items())
        .collect();
    let mut measured = measure_units(&units)?;
    for file in package.files() {
        let bound = measured
            .files
            .get_mut(file.module())
            .ok_or("missing C file measurement")?;
        syntax::account(file, bound)?;
    }
    let total = totals::sum(measured.files.values())?;
    measured.total.source_bound = total.source_bound;
    measured.total.max_identifier_bytes = total.max_identifier_bytes;
    Ok(measured)
}

pub(super) fn resolved(
    package: &portable_codegen::LinkedTargetPackage<super::CDialect>,
) -> Result<(), Vec<portable_diagnostics::Diagnostic>> {
    use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
    let source = SourceRef::logical(["c", "package"]);
    dependencies::verify(package).map_err(|message| {
        vec![Diagnostic::error(
            DiagnosticCode::TargetResourceLimit,
            message,
            source.clone(),
        )]
    })?;
    let measured = measure_package(package).map_err(|message| {
        vec![Diagnostic::error(
            DiagnosticCode::TargetResourceLimit,
            message,
            source.clone(),
        )]
    })?;
    let mut errors: Vec<_> = policy::check(&measured.total, source)
        .iter()
        .map(policy::CResourceError::diagnostic)
        .collect();
    for (file, measured) in &measured.files {
        errors.extend(
            policy::check(
                measured,
                SourceRef::logical(["c", file.key().path.as_str()]),
            )
            .iter()
            .map(policy::CResourceError::diagnostic),
        );
    }
    // Bound formatting work before allocating strings; then check the exact
    // formatter output against both per-file and aggregate source estimates.
    if !errors.is_empty() {
        return Err(errors);
    }
    syntax::verify_output(package, &measured).map_err(|message| {
        vec![Diagnostic::error(
            DiagnosticCode::TargetResourceLimit,
            message,
            SourceRef::logical(["c", "package"]),
        )]
    })
}

// Sum every object/value even across disjoint scopes. 16x layout storage covers
// copies/spills and alignment; each automatic object pays 256 bytes for
// instrumentation, each node 64 bytes, plus a fixed 4 KiB entry allowance.
// Pinned-compiler policy factors require native evidence, not an ABI theorem.
pub(super) fn frame_bound(measured: &Measurements) -> Result<u64, String> {
    measured
        .automatic_bytes
        .checked_add(measured.value_bytes)
        .and_then(|v| v.checked_mul(16))
        .and_then(|v| v.checked_add(measured.automatic_objects.checked_mul(256)?))
        .and_then(|v| v.checked_add(measured.nodes.checked_mul(64)?))
        .and_then(|v| v.checked_add(4096))
        .ok_or_else(|| "C frame bound overflow".into())
}
