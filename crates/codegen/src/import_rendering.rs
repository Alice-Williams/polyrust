//! Formatting contract for imports already allocated by the shared linker.
use crate::{LinkerDialect, ResolvedFileImport, ResolvedImport};

/// Spells the linked import list. Implementations may not discover dependencies,
/// allocate names, add bindings, or accept arbitrary source-text directives.
pub trait StructuralImportRenderer<D: LinkerDialect> {
    fn render_imports(
        &self,
        imports: &[ResolvedImport<D>],
        file_imports: &[ResolvedFileImport<D>],
    ) -> String;
}
