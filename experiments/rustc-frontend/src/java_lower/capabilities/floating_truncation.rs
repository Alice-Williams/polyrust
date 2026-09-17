//! Materialize once, then select exact catalogue-owned rounding calls.
use super::{FloatingTruncation, Mapping, TruncationInput};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::{
    ast::{
        JavaBinaryOperator, JavaCallableRef, JavaExpr, JavaExprKind, JavaLiteral, JavaPrecedence,
    },
    dialect::JavaKnownCallable,
};
#[derive(Clone, Copy)]
pub(crate) struct JavaFloatingTruncation;
impl Mapping for JavaFloatingTruncation {
    type Capability = FloatingTruncation;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: TruncationInput<'tcx>,
    ) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(truncation_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.receiver())?;
        let operand = reader.materialize(operand)?;
        if operand.plan() != &TypePlan::F64 {
            return Err("Floating truncation requires an exact Double receiver".into());
        }
        let operand = operand.into_expression();
        let call = |callable: JavaKnownCallable| JavaExpr {
            ty: TypePlan::F64.java_type(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Call {
                callable: JavaCallableRef::Known {
                    callable,
                    signature: callable.signature(),
                },
                receiver: None,
                arguments: vec![operand.clone()],
            },
        };
        let condition = JavaExpr {
            ty: TypePlan::Bool.java_type(),
            precedence: JavaPrecedence::Relational,
            kind: JavaExprKind::Binary {
                operator: JavaBinaryOperator::Less,
                left: Box::new(operand.clone()),
                right: Box::new(JavaExpr::literal(
                    TypePlan::F64.java_type(),
                    JavaLiteral::F64(
                        portable_binary64::FiniteBinary64::from_bits(0)
                            .expect("positive zero is finite"),
                    ),
                )),
            },
        };
        let result = Value::new(
            TypePlan::F64,
            JavaExpr {
                ty: TypePlan::F64.java_type(),
                precedence: JavaPrecedence::Conditional,
                kind: JavaExprKind::Conditional {
                    condition: Box::new(condition),
                    when_true: Box::new(call(JavaKnownCallable::MathCeil)),
                    when_false: Box::new(call(JavaKnownCallable::MathFloor)),
                },
            },
        )?;
        #[cfg(truncation_ast_probe)]
        super::truncation_ast::observe(reader, &input, &result.clone().into_expression(), start);
        Ok(result)
    }
}
