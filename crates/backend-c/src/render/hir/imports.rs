//! Only linker-owned imports reach directive spelling.
use super::super::CDialect;
use crate::dialect::CImportKind;
use portable_codegen::{ResolvedFileImport, ResolvedImport};
use std::fmt::Write;

pub(super) struct CImports;

impl ::portable_codegen::StructuralImportRenderer<CDialect> for CImports {
    fn render_imports(
        &self,
        imports: &[ResolvedImport<CDialect>],
        file_imports: &[ResolvedFileImport<CDialect>],
    ) -> String {
        let mut text = String::new();
        let mut seen = std::collections::BTreeSet::new();
        for kind in imports
            .iter()
            .map(ResolvedImport::kind)
            .chain(file_imports.iter().map(ResolvedFileImport::kind))
        {
            if !seen.insert(kind) {
                continue;
            }
            match kind {
                CImportKind::Standard(header) => writeln!(text, "#include <{}>", header.spelling()),
                CImportKind::Generated(header) => {
                    writeln!(text, "#include \"{}\"", header.include_path())
                }
                CImportKind::Dependency(package) => {
                    writeln!(
                        text,
                        "#include \"{}\"",
                        package.public_header().include_path()
                    )
                }
            }
            .expect("String formatting");
        }
        text
    }
}
