//! Block-local const declarations have compiler identity but no runtime storage.
use super::{Capability, LiteralValue, constant_evaluation};
use rustc_hir::{self as hir, def::DefKind, def_id::DefId};
use rustc_middle::ty::TyCtxt;

pub(crate) struct LocalConstants;

#[derive(Clone, Copy)]
pub(crate) struct LocalConstantInput<'tcx> {
    value: LiteralValue,
    _definition: DefId,
    _statement: &'tcx hir::Stmt<'tcx>,
}

impl Capability for LocalConstants {
    type Input<'tcx> = LocalConstantInput<'tcx>;
}

impl<'tcx> LocalConstantInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        statement: &'tcx hir::Stmt<'tcx>,
    ) -> Result<Self, String> {
        let hir::StmtKind::Item(item) = statement.kind else {
            return Err("local constant input requires an item statement".into());
        };
        let definition = tcx.hir_item(item).owner_id.def_id.to_def_id();
        if !matches!(
            tcx.def_kind(definition),
            DefKind::Const {
                is_type_const: false
            }
        ) {
            return Err("only scalar const item statements are implemented".into());
        }
        let (_, value) = constant_evaluation::evaluate(tcx, definition)?;
        Ok(Self {
            value,
            _definition: definition,
            _statement: statement,
        })
    }

    pub(crate) fn value(self) -> LiteralValue {
        self.value
    }

    #[cfg(local_constant_ast_probe)]
    pub(crate) fn origin(self) -> (DefId, &'tcx hir::Stmt<'tcx>) {
        (self._definition, self._statement)
    }
}
