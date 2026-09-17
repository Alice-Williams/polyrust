//! Compiler-authenticated public scalar declarations, independent of target syntax.
use super::{Capability, ScalarConstantValue, constant_evaluation};
use crate::source_origin::{
    identity,
    public_api::{DeclarationKind, Inventory},
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;

pub(crate) struct PublicConstants;

#[derive(Clone, Copy)]
pub(crate) struct ConstantDeclarationInput<'tcx> {
    tcx: TyCtxt<'tcx>,
    definition: DefId,
    value: ScalarConstantValue,
}

impl Capability for PublicConstants {
    type Input<'tcx> = ConstantDeclarationInput<'tcx>;
}

impl<'tcx> ConstantDeclarationInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        inventory: &Inventory,
        definition: DefId,
    ) -> Result<Self, String> {
        let declaration = inventory
            .declarations()
            .get(&identity(tcx, definition))
            .ok_or("public constant is absent from the compiler export inventory")?;
        if declaration.kind() != DeclarationKind::Constant
            || declaration.definition().to_def_id() != definition
        {
            return Err("public constant compiler identity or kind disagrees".into());
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
