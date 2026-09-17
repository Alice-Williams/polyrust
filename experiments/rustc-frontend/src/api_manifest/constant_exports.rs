//! Alias metadata must be the exact certificate-derived public binding inventory.
use portable_backend_c::dialect::{
    CDependencyApi, CDialect, CForeignConstantExport, c_source_package,
};
use portable_codegen::{RenderReadyPackage, RustCrateExports};
use std::sync::Arc;

pub(super) fn collect(
    package: &RenderReadyPackage<CDialect>,
    expected: &Arc<RustCrateExports>,
) -> Result<Vec<CForeignConstantExport>, String> {
    let Some(source) = c_source_package(package) else {
        return Ok(vec![]);
    };
    if source.exports() != expected {
        return Err(
            "API export graph differs from certified explicit source-package metadata".into(),
        );
    }
    // Only foreign views are retained. They preserve original producer authority;
    // the temporary facade API's independently minted owned handles never escape.
    if !expected
        .modules
        .values()
        .flat_map(|names| names.values())
        .any(|target| {
            let id = match target {
                portable_codegen::RustExportTarget::Module(id)
                | portable_codegen::RustExportTarget::Declaration(id) => id,
            };
            id.crate_id != expected.root.crate_id
        })
    {
        return Ok(vec![]);
    }
    Ok(CDependencyApi::from_certificate(package.clone())?
        .foreign_constants()
        .cloned()
        .collect())
}
