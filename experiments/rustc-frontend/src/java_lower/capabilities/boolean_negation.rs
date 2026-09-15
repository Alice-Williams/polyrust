//! bool Not uses an ordinary typed Java unary node, without helper calls.
use super::{BooleanNegation, Mapping, NegationInput};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaExpr, JavaExprKind, JavaPrecedence, JavaUnaryOperator};

#[derive(Clone, Copy)]
pub(crate) struct JavaBooleanNegation;

impl Mapping for JavaBooleanNegation {
    type Capability = BooleanNegation;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: NegationInput<'tcx>) -> Result<Value> {
        let operand = reader.expr(input.operand())?;
        if operand.plan() != &TypePlan::Bool {
            return Err("Boolean negation operand representation differs".into());
        }
        Value::new(
            TypePlan::Bool,
            JavaExpr {
                ty: TypePlan::Bool.java_type(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Unary {
                    operator: JavaUnaryOperator::Not,
                    operand: Box::new(operand.into_expression()),
                },
            },
        )
    }
}
