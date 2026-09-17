//! No wrapper/helper: compare two reads of one canonical receiver local.
use super::{FloatingNaN, Mapping, NaNInput};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaBinaryOperator, JavaExpr, JavaExprKind, JavaPrecedence};

#[derive(Clone, Copy)]
pub(crate) struct JavaFloatingNaN;
impl Mapping for JavaFloatingNaN {
    type Capability = FloatingNaN;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: NaNInput<'tcx>) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(nan_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.receiver())?;
        let operand = reader.materialize(operand)?;
        if operand.plan() != &TypePlan::F64 {
            return Err("NaN classification requires an exact Double receiver".into());
        }
        let operand = operand.into_expression();
        let result = Value::new(
            TypePlan::Bool,
            JavaExpr {
                ty: TypePlan::Bool.java_type(),
                precedence: JavaPrecedence::Equality,
                kind: JavaExprKind::Binary {
                    operator: JavaBinaryOperator::NotEqual,
                    left: Box::new(operand.clone()),
                    right: Box::new(operand),
                },
            },
        )?;
        #[cfg(nan_ast_probe)]
        super::nan_ast::observe(reader, &input, &result.clone().into_expression(), start);
        Ok(result)
    }
}
