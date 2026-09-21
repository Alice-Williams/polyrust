//! Ordered once-only primitive Double operands become structural remainder.
use super::{FloatingRemainder, Mapping, RemainderInput};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaBinaryOperator, JavaExpr, JavaExprKind, JavaPrecedence};

#[derive(Clone, Copy)]
pub(crate) struct JavaFloatingRemainder;
impl Mapping for JavaFloatingRemainder {
    type Capability = FloatingRemainder;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: RemainderInput<'tcx>) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(remainder_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        #[cfg(remainder_ast_probe)]
        let middle = reader.prelude.len();
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        if left.plan() != &TypePlan::F64 || right.plan() != left.plan() {
            return Err("floating remainder requires exact Double operands".into());
        }
        let result = Value::new(
            TypePlan::F64,
            JavaExpr {
                ty: TypePlan::F64.java_type(),
                precedence: JavaPrecedence::Multiplicative,
                kind: JavaExprKind::Binary {
                    operator: JavaBinaryOperator::Remainder,
                    left: Box::new(left.into_expression()),
                    right: Box::new(right.into_expression()),
                },
            },
        )?;
        #[cfg(remainder_ast_probe)]
        super::remainder_ast::observe(
            reader,
            &input,
            &result.clone().into_expression(),
            start,
            middle,
        );
        Ok(result)
    }
}
