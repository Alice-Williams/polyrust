//! Java primitive Boolean bitwise operators retain eager evaluation.
use super::{EagerBooleanInput, EagerBooleanOperator, EagerBooleans, Mapping};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaBinaryOperator, JavaExpr, JavaExprKind, JavaPrecedence};

#[derive(Clone, Copy)]
pub(crate) struct JavaEagerBooleans;

impl Mapping for JavaEagerBooleans {
    type Capability = EagerBooleans;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: EagerBooleanInput<'tcx>,
    ) -> Result<Value> {
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        if left.plan() != &TypePlan::Bool || right.plan() != &TypePlan::Bool {
            return Err("eager Boolean operand representations differ".into());
        }
        let (operator, precedence) = match input.operator() {
            EagerBooleanOperator::And => (JavaBinaryOperator::BitAnd, JavaPrecedence::BitAnd),
            EagerBooleanOperator::Or => (JavaBinaryOperator::BitOr, JavaPrecedence::BitOr),
            EagerBooleanOperator::Xor => (JavaBinaryOperator::BitXor, JavaPrecedence::BitXor),
        };
        let result = Value::new(
            TypePlan::Bool,
            JavaExpr {
                ty: TypePlan::Bool.java_type(),
                precedence,
                kind: JavaExprKind::Binary {
                    operator,
                    left: Box::new(left.into_expression()),
                    right: Box::new(right.into_expression()),
                },
            },
        )?;
        #[cfg(eager_ast_probe)]
        super::eager_ast::check(&input, &result.clone().into_expression());
        Ok(result)
    }
}
