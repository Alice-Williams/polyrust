//! Ordered original operands become primitive modular addition, without helpers.
use super::{AdditionInput, AdditionWidth, Mapping, WrappingAddition};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaBinaryOperator, JavaExpr, JavaExprKind, JavaPrecedence};

#[derive(Clone, Copy)]
pub(crate) struct JavaWrappingAddition;
impl Mapping for JavaWrappingAddition {
    type Capability = WrappingAddition;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: AdditionInput<'tcx>) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(addition_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        #[cfg(addition_ast_probe)]
        let middle = reader.prelude.len();
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        let plan = match input.width() {
            AdditionWidth::I32 => TypePlan::I32,
            AdditionWidth::I64 => TypePlan::I64,
        };
        if left.plan() != &plan || right.plan() != &plan {
            return Err("wrapping addition Java representation differs from checked width".into());
        }
        let result = Value::new(
            plan.clone(),
            JavaExpr {
                ty: plan.java_type(),
                precedence: JavaPrecedence::Additive,
                kind: JavaExprKind::Binary {
                    operator: JavaBinaryOperator::Add,
                    left: Box::new(left.into_expression()),
                    right: Box::new(right.into_expression()),
                },
            },
        )?;
        #[cfg(addition_ast_probe)]
        super::addition_ast::observe(
            reader,
            &input,
            &result.clone().into_expression(),
            start,
            middle,
        );
        Ok(result)
    }
}
