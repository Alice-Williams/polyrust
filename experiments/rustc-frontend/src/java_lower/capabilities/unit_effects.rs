//! Primitive void calls and structured effects have no Java value wrapper.
use super::{Mapping, UnitEffects, UnitInput, UnitOperation};
use crate::java_lower::{Reader, Result, TypePlan};
use portable_backend_java::ast::{JavaPrimitive, JavaStmt, JavaType};

#[cfg(unit_ast_probe)]
#[path = "../../../test/unit_java_ast.rs"]
pub(super) mod ast;

#[derive(Clone, Copy)]
pub(crate) struct JavaUnitEffects;
impl Mapping for JavaUnitEffects {
    type Capability = UnitEffects;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Vec<JavaStmt>;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: UnitInput<'tcx>,
    ) -> Result<Self::Output> {
        if reader.active_scope != Some(input.scope()) || !reader.prelude.is_empty() {
            return Err(
                "unit effects require the current source scope and an empty prelude".into(),
            );
        }
        let result: Result<Self::Output> = match input.operation() {
            UnitOperation::Empty => Ok(vec![]),
            UnitOperation::Call(source) => {
                let call = reader.prepare_direct_call(source)?;
                if call.ty != JavaType::primitive(JavaPrimitive::Void) {
                    return Err("unit call has a non-void registered result".into());
                }
                let mut statements = std::mem::take(&mut reader.prelude);
                statements.push(JavaStmt::Expression(call));
                Ok(statements)
            }
            UnitOperation::Block(expression) => {
                Ok(reader.effect_branch(expression, input.scope())?.statements)
            }
            UnitOperation::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                let condition = reader.expr(condition)?;
                if condition.plan() != &TypePlan::Bool {
                    return Err("unit conditional requires a Boolean condition".into());
                }
                let mut statements = std::mem::take(&mut reader.prelude);
                let then_block = reader.effect_branch(then_value, input.scope())?;
                let else_block = else_value
                    .map(|value| reader.effect_branch(value, input.scope()))
                    .transpose()?;
                statements.push(JavaStmt::If {
                    condition: condition.into_expression(),
                    then_block,
                    else_block,
                });
                Ok(statements)
            }
        };
        let statements = result?;
        #[cfg(unit_ast_probe)]
        ast::check(reader, &input, &statements);
        Ok(statements)
    }
}
