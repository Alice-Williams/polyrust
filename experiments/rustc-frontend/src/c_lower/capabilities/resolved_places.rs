//! Compiler identities, nominal fields and built-in shared dereference only.
use super::{Mapping, PlaceInput, ResolvedPlaces};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CAggregateRef, CObjectTypeKind, CPlace};
use rustc_hir::{self as hir, def::Res};
use rustc_middle::ty::{
    self,
    adjustment::{Adjust, DerefAdjustKind},
};

#[derive(Clone, Copy)]
pub(crate) struct CResolvedPlaces;

impl Mapping for CResolvedPlaces {
    type Capability = ResolvedPlaces;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CPlace;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: PlaceInput<'tcx>) -> Result<CPlace> {
        let expression = input.0;
        let source_type = reader.checked.expr_ty(expression);
        reader.ty(source_type)?;
        let mut result = match expression.kind {
            hir::ExprKind::Path(ref path) => {
                let Res::Local(id) = reader.checked.qpath_res(path, expression.hir_id) else {
                    return Err("only resolved local value paths are implemented".into());
                };
                reader
                    .bindings
                    .get(&id)
                    .ok_or("unregistered local binding")?
                    .clone()
            }
            hir::ExprKind::Field(base, _) => {
                let base = reader.place(base)?;
                let CObjectTypeKind::Struct(record) = base.ty().kind() else {
                    return Err("non-record field base".into());
                };
                let members = c(reader
                    .registry
                    .members(&CAggregateRef::Struct(record.clone())))?
                .ok_or("incomplete field owner")?;
                let member = members
                    .get(reader.checked.field_index(expression.hir_id).as_usize())
                    .ok_or("unknown field index")?
                    .clone();
                c(reader.expressions().member(base, member))?
            }
            hir::ExprKind::Unary(hir::UnOp::Deref, base)
                if matches!(
                    reader.checked.expr_ty(base).kind(),
                    ty::Ref(_, _, hir::Mutability::Not)
                ) && reader
                    .checked
                    .type_dependent_def_id(expression.hir_id)
                    .is_none() =>
            {
                let base = reader.place(base)?;
                let value = c(reader.expressions().read(base))?;
                c(reader.expressions().dereference(value))?
            }
            _ => return Err("only local/field/shared-dereference places are implemented".into()),
        };
        let mut adjusted_type = source_type;
        for adjustment in reader.checked.expr_adjustments(expression) {
            match adjustment.kind {
                Adjust::Deref(DerefAdjustKind::Builtin)
                    if matches!(adjusted_type.kind(), ty::Ref(_, _, hir::Mutability::Not)) =>
                {
                    let value = c(reader.expressions().read(result))?;
                    result = c(reader.expressions().dereference(value))?;
                    reader.ty(adjustment.target)?;
                    adjusted_type = adjustment.target;
                }
                _ => return Err("compiler adjustment is not implemented".into()),
            }
        }
        Ok(result)
    }
}
