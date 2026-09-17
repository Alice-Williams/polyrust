//! Foreign compiler declarations must still be joined to an original target owner.
use super::{Capability, PublicConstantReadInput, ScalarConstantValue, constant_evaluation};
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::DefId,
};
use rustc_middle::ty::{TyCtxt, TypeckResults};

pub(crate) struct PublicConstantImports;

#[derive(Clone, Copy)]
pub(crate) struct ConstantImportInput<'tcx> {
    tcx: TyCtxt<'tcx>,
    definition: DefId,
    value: ScalarConstantValue,
}

impl Capability for PublicConstantImports {
    type Input<'tcx> = ConstantImportInput<'tcx>;
}
impl<'tcx> ConstantImportInput<'tcx> {
    /// Discovery only; this does not authenticate a producer or create a read handle.
    pub(crate) fn discover(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Option<DefId> {
        let hir::ExprKind::Path(ref path) = expression.kind else {
            return None;
        };
        let Res::Def(
            DefKind::Const {
                is_type_const: false,
            },
            definition,
        ) = checked.qpath_res(path, expression.hir_id)
        else {
            return None;
        };
        (!definition.is_local() && PublicConstantReadInput::requires_reference(tcx, definition))
            .then_some(definition)
    }
    pub(crate) fn read(tcx: TyCtxt<'tcx>, definition: DefId) -> Result<Self, String> {
        if definition.is_local()
            || !PublicConstantReadInput::requires_reference(tcx, definition)
            || !tcx.visibility(definition).is_public()
        {
            return Err("constant import requires a public foreign module constant".into());
        }
        let (_, value) = constant_evaluation::evaluate(tcx, definition)?;
        Ok(Self {
            tcx,
            definition,
            value,
        })
    }
    pub(crate) fn tcx(self) -> TyCtxt<'tcx> {
        self.tcx
    }
    pub(crate) fn definition(self) -> DefId {
        self.definition
    }
    pub(crate) fn value(self) -> ScalarConstantValue {
        self.value
    }
}
