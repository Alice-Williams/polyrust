use super::{Mapping, RecordInitializers, RecordInput};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaConstructorRef, JavaExpr, JavaExprKind, JavaPrecedence};
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct JavaRecordInitializers;

impl Mapping for JavaRecordInitializers {
    type Capability = RecordInitializers;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: RecordInput<'tcx>) -> Result<Value> {
        let expression = input.0;
        let hir::ExprKind::Struct(_, fields, hir::StructTailExpr::None) = expression.kind else {
            return Err("only complete scalar-field record initializers are implemented".into());
        };
        if !reader.checked.expr_adjustments(expression).is_empty() {
            return Err("record initializer adjustment is not implemented".into());
        }
        let plan = reader.ty(reader.checked.expr_ty(expression))?;
        let TypePlan::Record(id) = plan else {
            return Err("non-record initializer".into());
        };
        let record = reader
            .records
            .values()
            .find(|record| record.id == id)
            .ok_or("unregistered record")?;
        let parameters: Vec<_> = record
            .declaration
            .record_components
            .iter()
            .map(|field| field.ty.clone())
            .collect();
        let mut arguments = vec![None; parameters.len()];
        for field in fields {
            let index = reader.checked.field_index(field.hir_id).as_usize();
            let slot = arguments
                .get_mut(index)
                .ok_or("unknown record member index")?;
            if slot.is_some() {
                return Err("duplicate record member".into());
            }
            let value = reader.initializer(field.expr)?;
            if value.plan() != &TypePlan::scalar(&parameters[index])? {
                return Err("record initializer type mismatch".into());
            }
            *slot = Some(reader.materialize(value)?.into_expression());
        }
        let arguments: Vec<JavaExpr> = arguments
            .into_iter()
            .map(|value| value.ok_or_else(|| "missing record member".to_owned()))
            .collect::<Result<_>>()?;
        #[cfg(java_ast_probe)]
        crate::java_lower::assertions::field_order(reader, fields, &arguments);
        Value::new(
            plan.clone(),
            JavaExpr {
                ty: plan.java_type(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::New {
                    constructor: JavaConstructorRef::Generated {
                        owner: id,
                        parameters,
                    },
                    arguments,
                },
            },
        )
    }
}
