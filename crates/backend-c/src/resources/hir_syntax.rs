//! Wrapper/import spelling is budgeted from typed witnesses, not source scans.
use super::super::{CDialect, CFileGrammar, CImportKind, spelling};
use super::{Measurements, PackageMeasurements, add};
use portable_codegen::{LinkedFile, LinkedTargetPackage};

pub(super) fn account(
    file: &LinkedFile<CDialect>,
    measured: &mut Measurements,
) -> Result<(), String> {
    // Guard text appears three times; fixed directive punctuation is bounded.
    // Directives are not executable AST nodes and do not invent stack frames.
    if let CFileGrammar::Header(guard) = file.source_kind() {
        let bytes = guard.identifier().as_str().len();
        measured.max_identifier_bytes = measured.max_identifier_bytes.max(bytes);
        add(
            &mut measured.source_bound,
            (bytes as u64)
                .checked_mul(3)
                .and_then(|value| value.checked_add(64))
                .ok_or("C guard byte overflow")?,
        )?;
    }
    for kind in file
        .imports()
        .iter()
        .map(|import| import.kind())
        .chain(file.file_imports().iter().map(|import| import.kind()))
    {
        let bytes = match kind {
            CImportKind::Standard(header) => header.spelling().len(),
            CImportKind::Generated(header) => header.include_path().len(),
            CImportKind::Dependency(package) => package.public_header().include_path().len(),
        };
        add(
            &mut measured.source_bound,
            (bytes as u64)
                .checked_add(12)
                .ok_or("C import byte overflow")?,
        )?;
    }
    if !file.imports().is_empty() || !file.file_imports().is_empty() {
        add(&mut measured.source_bound, 1)?;
    }
    Ok(())
}

pub(super) fn verify_output(
    package: &LinkedTargetPackage<CDialect>,
    measured: &PackageMeasurements,
) -> Result<(), String> {
    let mut bytes = 0;
    for file in package.files() {
        let output = spelling::file(file);
        let bound = measured
            .files
            .get(file.module())
            .ok_or("missing C output measurement")?;
        if output.len() as u64 > bound.source_bound {
            return Err("C formatted file exceeds its checked source estimate".into());
        }
        add(&mut bytes, output.len() as u64)?;
    }
    if bytes > measured.total.source_bound {
        return Err("C formatted package exceeds its checked source estimate".into());
    }
    Ok(())
}
