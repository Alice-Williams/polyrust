//! Ordered original operands become primitive modular subtraction, without helpers.
use super::{Mapping, SubtractionInput, SubtractionWidth, WrappingSubtraction};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaBinaryOperator, JavaExpr, JavaExprKind, JavaPrecedence};

#[derive(Clone, Copy)]
pub(crate) struct JavaWrappingSubtraction;
impl Mapping for JavaWrappingSubtraction {
    type Capability = WrappingSubtraction;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: SubtractionInput<'tcx>,
    ) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(subtraction_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        #[cfg(subtraction_ast_probe)]
        let middle = reader.prelude.len();
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        let plan = match input.width() {
            SubtractionWidth::I32 => TypePlan::I32,
            SubtractionWidth::I64 => TypePlan::I64,
        };
        if left.plan() != &plan || right.plan() != &plan {
            return Err(
                "wrapping subtraction Java representation differs from checked width".into(),
            );
        }
        let result = Value::new(
            plan.clone(),
            JavaExpr {
                ty: plan.java_type(),
                precedence: JavaPrecedence::Additive,
                kind: JavaExprKind::Binary {
                    operator: JavaBinaryOperator::Subtract,
                    left: Box::new(left.into_expression()),
                    right: Box::new(right.into_expression()),
                },
            },
        )?;
        #[cfg(subtraction_ast_probe)]
        super::subtraction_ast::observe(
            reader,
            &input,
            &result.clone().into_expression(),
            start,
            middle,
        );
        Ok(result)
    }
}
