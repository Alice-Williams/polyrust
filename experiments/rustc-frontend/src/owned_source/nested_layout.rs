//! Bounded compiler nominal metadata; no reconstructed source-language types.
use super::NestedError as E;
use rustc_abi::FieldIdx;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self, AdtDef, GenericArgsRef, Ty, TyCtxt};
use std::collections::HashSet;

pub(crate) enum FieldKind<'tcx> {
    Box(Ty<'tcx>),
    Record(RecordLayout<'tcx>),
}
pub(crate) struct Field<'tcx> {
    index: FieldIdx,
    declaration: DefId,
    kind: FieldKind<'tcx>,
}
impl<'tcx> Field<'tcx> {
    pub(crate) fn index(&self) -> FieldIdx {
        self.index
    }
    pub(crate) fn declaration(&self) -> DefId {
        self.declaration
    }
    pub(crate) fn kind(&self) -> &FieldKind<'tcx> {
        &self.kind
    }
    pub(crate) fn ty(&self) -> Ty<'tcx> {
        match &self.kind {
            FieldKind::Box(ty) => *ty,
            FieldKind::Record(record) => record.ty(),
        }
    }
}
pub(crate) struct RecordLayout<'tcx> {
    definition: AdtDef<'tcx>,
    arguments: GenericArgsRef<'tcx>,
    ty: Ty<'tcx>,
    fields: Vec<Field<'tcx>>,
}
impl<'tcx> RecordLayout<'tcx> {
    pub(crate) fn definition(&self) -> AdtDef<'tcx> {
        self.definition
    }
    pub(crate) fn arguments(&self) -> GenericArgsRef<'tcx> {
        self.arguments
    }
    pub(crate) fn ty(&self) -> Ty<'tcx> {
        self.ty
    }
    pub(crate) fn fields(&self) -> &[Field<'tcx>] {
        &self.fields
    }
}
pub(super) fn read<'tcx>(tcx: TyCtxt<'tcx>, result: Ty<'tcx>) -> Result<RecordLayout<'tcx>, E> {
    let box_ty = super::super::local_call::scalar_box_type(tcx).ok_or(E::UnsupportedField)?;
    let mut reader = Reader {
        tcx,
        box_ty,
        ancestors: HashSet::new(),
        remaining: 128,
    };
    reader.record(result, 1)
}
struct Reader<'tcx> {
    tcx: TyCtxt<'tcx>,
    box_ty: Ty<'tcx>,
    ancestors: HashSet<DefId>,
    remaining: usize,
}
impl<'tcx> Reader<'tcx> {
    fn record(&mut self, result: Ty<'tcx>, depth: usize) -> Result<RecordLayout<'tcx>, E> {
        if depth > 8 {
            return Err(E::DepthBudget);
        }
        let ty::Adt(definition, arguments) = *result.kind() else {
            return Err(E::UnsupportedRecord);
        };
        if !definition.did().is_local()
            || !definition.is_struct()
            || !arguments.is_empty()
            || self.tcx.generics_of(definition.did()).count() != 0
            || definition.has_dtor(self.tcx)
        {
            return Err(E::UnsupportedRecord);
        }
        let repr = definition.repr();
        if repr.int.is_some()
            || repr.align.is_some()
            || repr.pack.is_some()
            || repr.scalable.is_some()
            || !repr.flags.is_empty()
        {
            return Err(E::UnsupportedRecord);
        }
        let variant = definition.non_enum_variant();
        if variant.ctor_kind().is_some() || variant.fields.is_empty() {
            return Err(E::UnsupportedRecord);
        }
        if variant.fields.len() > self.remaining {
            return Err(E::FieldBudget);
        }
        self.remaining -= variant.fields.len();
        if !self.ancestors.insert(definition.did()) {
            return Err(E::RecursiveRecord);
        }
        let mut fields = Vec::new();
        for (index, declaration) in variant.fields.iter_enumerated() {
            let ty = self
                .tcx
                .try_normalize_erasing_regions(
                    ty::TypingEnv::fully_monomorphized(),
                    declaration.ty(self.tcx, arguments),
                )
                .map_err(|_| E::UnsupportedField)?;
            let kind = self.field_kind(ty, depth)?;
            fields.push(Field {
                index,
                declaration: declaration.did,
                kind,
            });
        }
        self.ancestors.remove(&definition.did());
        Ok(RecordLayout {
            definition,
            arguments,
            ty: result,
            fields,
        })
    }
    fn field_kind(&mut self, ty: Ty<'tcx>, depth: usize) -> Result<FieldKind<'tcx>, E> {
        if ty == self.box_ty {
            return Ok(FieldKind::Box(ty));
        }
        match ty.kind() {
            ty::Adt(definition, _)
                if Some(definition.did()) != self.tcx.lang_items().owned_box() =>
            {
                Ok(FieldKind::Record(self.record(ty, depth + 1)?))
            }
            _ => Err(E::UnsupportedField),
        }
    }
}

#[cfg(nested_record_proof)]
#[path = "../../test/nested_record/layout_oracles.rs"]
pub(crate) mod oracles;
