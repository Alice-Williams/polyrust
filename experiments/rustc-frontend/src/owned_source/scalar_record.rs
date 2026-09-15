//! Closed scalar payload metadata, independent of record/Box producer paths.
use rustc_abi::FieldIdx;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self, AdtDef, GenericArgsRef, Ty, TyCtxt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalarKind {
    I32,
    Bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PayloadError {
    NotRecord,
    UnsupportedRecord,
    UnsupportedField,
}
pub(crate) struct ScalarField<'tcx> {
    index: FieldIdx,
    declaration: DefId,
    ty: Ty<'tcx>,
    kind: ScalarKind,
}
impl<'tcx> ScalarField<'tcx> {
    pub(crate) fn index(&self) -> FieldIdx {
        self.index
    }
    pub(crate) fn declaration(&self) -> DefId {
        self.declaration
    }
    pub(crate) fn ty(&self) -> Ty<'tcx> {
        self.ty
    }
    pub(crate) fn kind(&self) -> ScalarKind {
        self.kind
    }
}
pub(crate) struct ScalarRecord<'tcx> {
    definition: AdtDef<'tcx>,
    arguments: GenericArgsRef<'tcx>,
    ty: Ty<'tcx>,
    fields: Vec<ScalarField<'tcx>>,
}
impl<'tcx> ScalarRecord<'tcx> {
    pub(super) fn read(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Result<Self, PayloadError> {
        let &ty::Adt(definition, arguments) = ty.kind() else {
            return Err(PayloadError::NotRecord);
        };
        if !definition.did().is_local()
            || !definition.is_struct()
            || !arguments.is_empty()
            || tcx.generics_of(definition.did()).count() != 0
            || definition.has_dtor(tcx)
        {
            return Err(PayloadError::UnsupportedRecord);
        }
        let repr = definition.repr();
        if repr.int.is_some()
            || repr.align.is_some()
            || repr.pack.is_some()
            || repr.scalable.is_some()
            || !repr.flags.is_empty()
        {
            return Err(PayloadError::UnsupportedRecord);
        }
        let variant = definition.non_enum_variant();
        if variant.ctor_kind().is_some() || variant.fields.is_empty() || variant.fields.len() > 128
        {
            return Err(PayloadError::UnsupportedRecord);
        }
        let mut fields = Vec::new();
        for (index, field) in variant.fields.iter_enumerated() {
            let ty = tcx
                .try_normalize_erasing_regions(
                    ty::TypingEnv::fully_monomorphized(),
                    field.ty(tcx, arguments),
                )
                .map_err(|_| PayloadError::UnsupportedField)?;
            let kind = match ty.kind() {
                ty::Int(ty::IntTy::I32) => ScalarKind::I32,
                ty::Bool => ScalarKind::Bool,
                _ => return Err(PayloadError::UnsupportedField),
            };
            fields.push(ScalarField {
                index,
                declaration: field.did,
                ty,
                kind,
            });
        }
        Ok(Self {
            definition,
            arguments,
            ty,
            fields,
        })
    }
    pub(crate) fn definition(&self) -> AdtDef<'tcx> {
        self.definition
    }
    pub(crate) fn arguments(&self) -> GenericArgsRef<'tcx> {
        self.arguments
    }
    pub(crate) fn ty(&self) -> Ty<'tcx> {
        self.ty
    }
    pub(crate) fn fields(&self) -> &[ScalarField<'tcx>] {
        &self.fields
    }
}
