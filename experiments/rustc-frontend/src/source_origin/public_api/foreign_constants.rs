//! Compiler facts for a foreign export, never target or producer authority.
use rustc_hir::{def::DefKind, def_id::DefId};
use rustc_middle::ty::{TyCtxt, Visibility};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ForeignConstantDeclaration {
    definition: DefId,
}
impl ForeignConstantDeclaration {
    pub(super) fn read(tcx: TyCtxt<'_>, definition: DefId) -> Result<Self, String> {
        if definition.is_local()
            || tcx.def_kind(definition)
                != (DefKind::Const {
                    is_type_const: false,
                })
            || tcx.def_kind(tcx.parent(definition)) != DefKind::Mod
            || tcx.visibility(definition) != Visibility::Public
        {
            return Err(
                "foreign public binding requires an ordinary public module constant".into(),
            );
        }
        Ok(Self { definition })
    }

    pub(crate) fn definition(self) -> DefId {
        self.definition
    }
}

#[derive(Clone, Copy)]
pub(super) enum Policy {
    OwnedOnly,
    ForeignConstants,
}
