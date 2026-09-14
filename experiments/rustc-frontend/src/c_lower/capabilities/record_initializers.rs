//! Source-ordered evaluation precedes compiler-indexed field arrangement.
use super::{Mapping, RecordInitializers, RecordInput};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CAggregateRef, CInitializer, CObjectTypeKind};
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct CRecordInitializers;
impl Mapping for CRecordInitializers {
    type Capability = RecordInitializers;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CInitializer;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: RecordInput<'tcx>,
    ) -> Result<CInitializer> {
        let expression = input.0;
        let hir::ExprKind::Struct(_, fields, hir::StructTailExpr::None) = expression.kind else {
            return Err("only complete scalar-field record initializers are implemented".into());
        };
        if !reader.checked.expr_adjustments(expression).is_empty() {
            return Err("record initializer adjustment is not implemented".into());
        }
        let ty = reader.ty(reader.checked.expr_ty(expression))?;
        let CObjectTypeKind::Struct(record) = ty.kind() else {
            return Err("non-record initializer".into());
        };
        let members = c(reader
            .registry
            .members(&CAggregateRef::Struct(record.clone())))?
        .ok_or("incomplete record")?
        .to_vec();
        let mut values = vec![None; members.len()];
        // Calls materialize their arguments/results into the current prelude
        // in source field order. Only pure remaining values are then arranged
        // by member index; the enclosing let drains the prelude before storage.
        for field in fields {
            let index = reader.checked.field_index(field.hir_id).as_usize();
            let slot = values.get_mut(index).ok_or("unknown record member index")?;
            if slot.is_some() {
                return Err("duplicate record member".into());
            }
            *slot = Some(reader.initializer(field.expr)?);
        }
        let members = members
            .into_iter()
            .zip(values)
            .map(|(member, value)| Ok((member, value.ok_or("missing record member")?)))
            .collect::<Result<_>>()?;
        c(reader
            .expressions()
            .struct_initializer(record.clone(), members))
    }
}
