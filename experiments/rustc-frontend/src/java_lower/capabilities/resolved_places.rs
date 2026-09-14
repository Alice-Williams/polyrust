use super::{Mapping, PlaceInput, ResolvedPlaces};
use crate::java_lower::{Place, Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaExpr, JavaExprKind, JavaPrecedence};
use rustc_hir::{self as hir, def::Res};
use rustc_middle::ty::{
    self,
    adjustment::{Adjust, DerefAdjustKind},
};

#[derive(Clone, Copy)]
pub(crate) struct JavaResolvedPlaces;

impl Mapping for JavaResolvedPlaces {
    type Capability = ResolvedPlaces;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Place;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: PlaceInput<'tcx>) -> Result<Place> {
        let expression = input.0;
        let source_type = reader.checked.expr_ty(expression);
        let expected = reader.ty(source_type)?;
        let mut result = match expression.kind {
            hir::ExprKind::Path(ref path) => {
                let Res::Local(id) = reader.checked.qpath_res(path, expression.hir_id) else {
                    return Err("only resolved local value paths are implemented".into());
                };
                reader
                    .bindings
                    .get(&id)
                    .cloned()
                    .ok_or("unregistered local binding")?
            }
            hir::ExprKind::Field(base, _) => {
                let base = reader.place(base)?;
                let TypePlan::Record(id) = base.plan() else {
                    return Err("non-record field base".into());
                };
                let record = reader
                    .records
                    .values()
                    .find(|record| record.id == *id)
                    .ok_or("unregistered record")?;
                let index = reader.checked.field_index(expression.hir_id).as_usize();
                let reference = record
                    .fields
                    .get(index)
                    .ok_or("unknown field index")?
                    .clone();
                let plan = TypePlan::scalar(&record.declaration.record_components[index].ty)?;
                Place::resolved(Value::new(
                    plan.clone(),
                    JavaExpr {
                        ty: plan.java_type(),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Field {
                            receiver: Box::new(base.value().into_expression()),
                            field: reference,
                        },
                    },
                )?)
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
                reader.place(base)?.dereference()?
            }
            _ => return Err("only local/field/shared-dereference places are implemented".into()),
        };
        if result.plan() != &expected {
            return Err("source place representation mismatch".into());
        }
        let mut adjusted = source_type;
        for adjustment in reader.checked.expr_adjustments(expression) {
            match adjustment.kind {
                Adjust::Deref(DerefAdjustKind::Builtin)
                    if matches!(adjusted.kind(), ty::Ref(_, _, hir::Mutability::Not)) =>
                {
                    result = result.dereference()?;
                    let expected = reader.ty(adjustment.target)?;
                    if result.plan() != &expected {
                        return Err("adjusted place representation mismatch".into());
                    }
                    adjusted = adjustment.target;
                }
                _ => return Err("compiler adjustment is not implemented".into()),
            }
        }
        Ok(result)
    }
}
