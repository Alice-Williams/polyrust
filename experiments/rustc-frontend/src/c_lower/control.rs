//! Lexical traversal invokes the registered structured-control mapping.
use super::{
    Reader, Result,
    capabilities::{ControlInput, LexicalControl, Mapping, Supports},
};
use portable_backend_c::ast::CBlock;
use rustc_hir as hir;
impl<'tcx> Reader<'tcx> {
    pub(super) fn branch(
        &mut self,
        value: &'tcx hir::Expr<'tcx>,
        parent: Option<hir::HirId>,
    ) -> Result<CBlock> {
        let mapping = Supports::<LexicalControl>::mapping(&self.mappings);
        mapping.lower(
            self,
            ControlInput {
                expression: value,
                parent,
            },
        )
    }
}
