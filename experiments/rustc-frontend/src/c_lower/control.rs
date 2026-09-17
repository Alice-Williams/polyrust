//! Lexical traversal invokes the registered structured-control mapping.
use super::{
    Reader, Result,
    capabilities::{
        ControlCompletion, ControlInput, LexicalControl, Mapping, Supports, UnitEffects, UnitInput,
    },
};
use portable_backend_c::ast::{CBlock, CStatement};
use rustc_hir as hir;
impl<'tcx> Reader<'tcx> {
    pub(super) fn branch(
        &mut self,
        value: &'tcx hir::Expr<'tcx>,
        parent: Option<hir::HirId>,
    ) -> Result<CBlock> {
        self.control(value, parent, ControlCompletion::Return)
    }

    pub(super) fn effect_branch(
        &mut self,
        value: &'tcx hir::Expr<'tcx>,
        parent: hir::HirId,
    ) -> Result<CBlock> {
        self.control(value, Some(parent), ControlCompletion::Effect)
    }

    pub(super) fn unit(
        &mut self,
        value: &'tcx hir::Expr<'tcx>,
        scope: hir::HirId,
    ) -> Result<Vec<CStatement>> {
        let input = UnitInput::read(self.checked, value, scope)?;
        Supports::<UnitEffects>::mapping(&self.mappings).lower(self, input)
    }

    fn control(
        &mut self,
        value: &'tcx hir::Expr<'tcx>,
        parent: Option<hir::HirId>,
        completion: ControlCompletion,
    ) -> Result<CBlock> {
        let mapping = Supports::<LexicalControl>::mapping(&self.mappings);
        mapping.lower(
            self,
            ControlInput {
                expression: value,
                parent,
                completion,
            },
        )
    }
}
