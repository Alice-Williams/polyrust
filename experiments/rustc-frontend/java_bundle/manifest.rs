//! Borrowed source facts: these private values cannot become callable authority.
use crate::Owner;
use portable_backend_java::dialect::JavaSourceDescription;
use portable_codegen::{RustCrateExports, RustDeclarationId, RustModuleDocumentation};
use std::collections::BTreeMap;

pub(crate) struct Manifest<'a> {
    pub(crate) owner: Owner<'a>,
    pub(crate) source: String,
    pub(crate) filename: String,
    pub(crate) declarations: Vec<JavaSourceDescription<'a>>,
    pub(crate) modules: BTreeMap<RustDeclarationId, &'a RustModuleDocumentation>,
    pub(crate) exports: &'a RustCrateExports,
    pub(crate) source_bound: u64,
    pub(crate) json_bound: u64,
}

impl Manifest<'_> {
    pub(crate) fn verify_owner(&self) -> Result<(), String> {
        let expected = crate::projection::project(self.owner)?;
        if self.source != expected.source
            || self.filename != expected.filename
            || self.source_bound != expected.source_bound
            || !std::ptr::eq(self.exports, expected.exports)
            || self.declarations.len() != expected.declarations.len()
            || !self
                .declarations
                .iter()
                .zip(&expected.declarations)
                .all(|(a, b)| {
                    std::ptr::eq(a.source(), b.source())
                        && a.target() == b.target()
                        && a.kind() == b.kind()
                })
            || self.modules.len() != expected.modules.len()
            || !self
                .modules
                .iter()
                .zip(&expected.modules)
                .all(|((aid, a), (bid, b))| aid == bid && std::ptr::eq(*a, *b))
        {
            return Err("Java manifest differs from exact owner inventory".into());
        }
        Ok(())
    }
}
