//! Primitive Int-to-Long cast after materializing the original operand once.
use super::{Mapping, SignedWidening, WideningInput};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaExpr, JavaExprKind, JavaPrecedence};

#[derive(Clone, Copy)]
pub(crate) struct JavaSignedWidening;
impl Mapping for JavaSignedWidening {
    type Capability = SignedWidening;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: WideningInput<'tcx>) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(widening_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.operand())?;
        let operand = reader.materialize(operand)?;
        if operand.plan() != &TypePlan::I32 {
            return Err("signed widening Java representation differs from checked i32".into());
        }
        let result = Value::new(
            TypePlan::I64,
            JavaExpr {
                ty: TypePlan::I64.java_type(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Cast {
                    target: TypePlan::I64.java_type(),
                    value: Box::new(operand.into_expression()),
                },
            },
        )?;
        #[cfg(widening_ast_probe)]
        super::widening_ast::check(reader, &input, &result.clone().into_expression(), start);
        Ok(result)
    }
}
