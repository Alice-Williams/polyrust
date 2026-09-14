use super::{LiteralInput, LiteralValues, Mapping};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaExpr, JavaLiteral};
use rustc_ast::LitKind;
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct JavaLiteralValues;

impl Mapping for JavaLiteralValues {
    type Capability = LiteralValues;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: LiteralInput<'tcx>) -> Result<Value> {
        let expression = input.0;
        if !reader.checked.expr_adjustments(expression).is_empty() {
            return Err("literal compiler adjustment is not implemented".into());
        }
        let (plan, literal) = match expression.kind {
            hir::ExprKind::Lit(literal) => match literal.node {
                LitKind::Int(magnitude, _)
                    if reader.checked.expr_ty(expression) == reader.tcx.types.i32 =>
                {
                    (
                        TypePlan::I32,
                        JavaLiteral::I32(
                            i32::try_from(magnitude.0).map_err(|_| "i32 literal overflow")?,
                        ),
                    )
                }
                LitKind::Bool(value)
                    if reader.checked.expr_ty(expression) == reader.tcx.types.bool =>
                {
                    (TypePlan::Bool, JavaLiteral::Boolean(value))
                }
                _ => return Err("literal is not implemented".into()),
            },
            hir::ExprKind::Unary(hir::UnOp::Neg, operand)
                if reader.checked.expr_ty(expression) == reader.tcx.types.i32 =>
            {
                let hir::ExprKind::Lit(literal) = operand.kind else {
                    return Err("only negative integer literals are implemented".into());
                };
                let LitKind::Int(magnitude, _) = literal.node else {
                    return Err("only negative integer literals are implemented".into());
                };
                let magnitude =
                    i64::try_from(magnitude.0).map_err(|_| "integer literal overflow")?;
                (
                    TypePlan::I32,
                    JavaLiteral::I32(
                        i32::try_from(-magnitude).map_err(|_| "i32 literal overflow")?,
                    ),
                )
            }
            _ => return Err("literal capability received an unsupported source shape".into()),
        };
        Value::new(plan.clone(), JavaExpr::literal(plan.java_type(), literal))
    }
}
