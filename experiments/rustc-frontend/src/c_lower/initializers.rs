//! Select the registered record mapping or an ordinary expression initializer.
use super::{
    Reader, Result, c,
    capabilities::{Mapping, RecordInitializers, RecordInput, Supports},
};
use portable_backend_c::ast::CInitializer;
use rustc_hir as hir;

impl<'tcx> Reader<'tcx> {
    pub(super) fn initializer(
        &mut self,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<CInitializer> {
        if matches!(expression.kind, hir::ExprKind::Struct(..)) {
            let mapping = Supports::<RecordInitializers>::mapping(&self.mappings);
            mapping.lower(self, RecordInput(expression))
        } else {
            let value = self.expr(expression)?;
            c(self.expressions().expression_initializer(value))
        }
    }
}
