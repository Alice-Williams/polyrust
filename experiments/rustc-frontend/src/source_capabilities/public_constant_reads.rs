//! Public package reads retain a checked definition rather than choosing a name.
use super::{Capability, ConstantInput, LiteralValue};
use rustc_hir::{self as hir, def::DefKind, def_id::DefId};
use rustc_middle::ty::{TyCtxt, TypeckResults};

pub(crate) struct PublicConstantReads;

#[derive(Clone, Copy)]
pub(crate) struct PublicConstantReadInput<'tcx> {
    checked: ConstantInput<'tcx>,
}
impl Capability for PublicConstantReads {
    type Input<'tcx> = PublicConstantReadInput<'tcx>;
}
impl<'tcx> PublicConstantReadInput<'tcx> {
    pub(crate) fn requires_reference(tcx: TyCtxt<'tcx>, definition: DefId) -> bool {
        matches!(
            tcx.def_kind(definition),
            DefKind::Const {
                is_type_const: false
            }
        ) && tcx.def_kind(tcx.parent(definition)) == DefKind::Mod
            && definition
                .as_local()
                .is_none_or(|id| tcx.effective_visibilities(()).is_exported(id))
    }
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let checked = ConstantInput::read(tcx, checked, expression)?;
        if !Self::requires_reference(tcx, checked.definition()) {
            return Err("public constant read requires an exported module constant".into());
        }
        // This is compiler provenance, not target import authority. Both executable
        // target mappings require a pre-registered opaque producer witness for
        // every foreign read; missing/stale identities and values remain errors.
        Ok(Self { checked })
    }
    pub(crate) fn definition(self) -> DefId {
        self.checked.definition()
    }
    pub(crate) fn value(self) -> LiteralValue {
        self.checked.value()
    }
}
