//! Nominal source-place metadata, not an independently constructible body proof.
use super::super::{LinearError as E, Result, source::local};
use rustc_abi::FieldIdx;
use rustc_hir::{self as hir, HirId, def_id::DefId};
use rustc_middle::{
    mir,
    ty::{self, Ty, TyCtxt, TypeckResults},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Field<'tcx> {
    parent: Ty<'tcx>,
    declaration: DefId,
    index: FieldIdx,
    ty: Ty<'tcx>,
}
impl<'tcx> Field<'tcx> {
    pub(crate) fn parent(&self) -> Ty<'tcx> {
        self.parent
    }
    pub(crate) fn declaration(&self) -> DefId {
        self.declaration
    }
    pub(crate) fn index(&self) -> FieldIdx {
        self.index
    }
    pub(crate) fn ty(&self) -> Ty<'tcx> {
        self.ty
    }
}
pub(super) fn field<'tcx>(
    tcx: TyCtxt<'tcx>,
    parent: Ty<'tcx>,
    index: FieldIdx,
) -> Result<Field<'tcx>> {
    let ty::Adt(definition, arguments) = parent.kind() else {
        return Err(E::SourceIdentity);
    };
    if !definition.is_struct() {
        return Err(E::SourceIdentity);
    }
    let declared = definition
        .non_enum_variant()
        .fields
        .get(index)
        .ok_or(E::SourceIdentity)?;
    let ty = tcx
        .try_normalize_erasing_regions(
            ty::TypingEnv::fully_monomorphized(),
            declared.ty(tcx, arguments),
        )
        .map_err(|_| E::SourceIdentity)?;
    Ok(Field {
        parent,
        declaration: declared.did,
        index,
        ty,
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SourcePlace<'tcx> {
    binding: HirId,
    fields: Vec<Field<'tcx>>,
}
impl<'tcx> SourcePlace<'tcx> {
    pub(super) fn local(binding: HirId) -> Self {
        Self {
            binding,
            fields: Vec::new(),
        }
    }
    pub(super) fn child(&self, field: Field<'tcx>) -> Self {
        let mut next = self.clone();
        next.fields.push(field);
        next
    }
    pub(crate) fn binding(&self) -> HirId {
        self.binding
    }
    pub(crate) fn fields(&self) -> &[Field<'tcx>] {
        &self.fields
    }
    pub(super) fn parent(&self) -> Option<Self> {
        let mut result = self.clone();
        result.fields.pop()?;
        Some(result)
    }
    pub(super) fn contains(&self, other: &Self) -> bool {
        self.binding == other.binding && other.fields.starts_with(&self.fields)
    }
    pub(super) fn relocate(&self, from: &Self, to: &Self) -> Result<Self> {
        if !from.contains(self) {
            return Err(E::SourceIdentity);
        }
        let mut fields = to.fields.clone();
        fields.extend_from_slice(&self.fields[from.fields.len()..]);
        if fields.len() > 2 {
            return Err(E::Budget);
        }
        Ok(Self {
            binding: to.binding,
            fields,
        })
    }
    pub(super) fn project(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &mir::Body<'tcx>,
        root: mir::Local,
    ) -> Result<mir::Place<'tcx>> {
        let mut place = mir::Place::from(root);
        for field in &self.fields {
            if place.ty(&body.local_decls, tcx).ty != field.parent {
                return Err(E::MoveGraph);
            }
            place = place.project_deeper(&[mir::ProjectionElem::Field(field.index, field.ty)], tcx);
        }
        Ok(place)
    }
}
pub(super) fn read<'tcx>(
    tcx: TyCtxt<'tcx>,
    checked: &TypeckResults<'tcx>,
    expression: &'tcx hir::Expr<'tcx>,
) -> Result<SourcePlace<'tcx>> {
    let mut projections = Vec::new();
    let mut cursor = expression;
    while let hir::ExprKind::Field(base, _) = cursor.kind {
        if projections.len() == 2
            || !checked.expr_adjustments(cursor).is_empty()
            || !checked.expr_adjustments(base).is_empty()
        {
            return Err(E::BodyShape);
        }
        let field = field(
            tcx,
            checked.expr_ty(base),
            checked.field_index(cursor.hir_id),
        )?;
        if checked.expr_ty(cursor) != field.ty {
            return Err(E::SourceIdentity);
        }
        projections.push(field);
        cursor = base;
    }
    let binding = local(checked, cursor)?;
    projections.reverse();
    Ok(SourcePlace {
        binding,
        fields: projections,
    })
}
