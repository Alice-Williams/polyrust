//! Primitive typed selection; no Math import, wrapper or runtime helper.
use super::{AbsoluteInput, FloatingAbsolute, Mapping};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{
    JavaBinaryOperator, JavaExpr, JavaExprKind, JavaLiteral, JavaPrecedence, JavaUnaryOperator,
};
#[derive(Clone, Copy)]
pub(crate) struct JavaFloatingAbsolute;

fn conditional(condition: JavaExpr, when_true: JavaExpr, when_false: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: TypePlan::F64.java_type(),
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        },
    }
}
impl Mapping for JavaFloatingAbsolute {
    type Capability = FloatingAbsolute;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: AbsoluteInput<'tcx>) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(absolute_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.receiver())?;
        let operand = reader.materialize(operand)?;
        if operand.plan() != &TypePlan::F64 {
            return Err("Floating absolute value requires an exact Double receiver".into());
        }
        let operand = operand.into_expression();
        let zero = JavaExpr {
            ty: TypePlan::F64.java_type(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Literal(JavaLiteral::F64(
                portable_binary64::FiniteBinary64::from_bits(0).expect("positive zero is finite"),
            )),
        };
        let comparison = |operator, precedence| JavaExpr {
            ty: TypePlan::Bool.java_type(),
            precedence,
            kind: JavaExprKind::Binary {
                operator,
                left: Box::new(operand.clone()),
                right: Box::new(zero.clone()),
            },
        };
        let below = comparison(JavaBinaryOperator::Less, JavaPrecedence::Relational);
        let equal = comparison(JavaBinaryOperator::Equal, JavaPrecedence::Equality);
        let negative = JavaExpr {
            ty: TypePlan::F64.java_type(),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Unary {
                operator: JavaUnaryOperator::Negate,
                operand: Box::new(operand.clone()),
            },
        };
        let magnitude = conditional(below, negative, operand);
        let result = Value::new(TypePlan::F64, conditional(equal, zero, magnitude))?;
        #[cfg(absolute_ast_probe)]
        super::absolute_ast::observe(reader, &input, &result.clone().into_expression(), start);
        Ok(result)
    }
}
