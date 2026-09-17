//! Unit lowers to ordinary typed effects and blocks, never a C object.
use super::{Mapping, UnitEffects, UnitInput, UnitOperation};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::*;

#[cfg(unit_ast_probe)]
#[path = "../../../test/unit_c_ast.rs"]
pub(super) mod ast;

#[derive(Clone, Copy)]
pub(crate) struct CUnitEffects;
impl Mapping for CUnitEffects {
    type Capability = UnitEffects;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Vec<CStatement>;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: UnitInput<'tcx>,
    ) -> Result<Self::Output> {
        if !reader.prelude.is_empty()
            || reader.active_scope.is_none()
            || reader.control_scopes.get(&input.scope()) != reader.active_scope.as_ref()
        {
            return Err(
                "unit effects require the current source scope and an empty prelude".into(),
            );
        }
        let result: Result<Self::Output> = match input.operation() {
            UnitOperation::Empty => Ok(vec![]),
            UnitOperation::Call(expression) => {
                let (function, arguments) = reader.prepare_direct_call(expression)?;
                let callable = c(reader.expressions().direct(function))?;
                let effect = c(reader.expressions().call_effect(callable, arguments))?;
                let mut statements = std::mem::take(&mut reader.prelude);
                statements.push(c(reader.statements()?.evaluate(effect))?);
                Ok(statements)
            }
            UnitOperation::Block(expression) => {
                let block = reader.effect_branch(expression, input.scope())?;
                Ok(vec![c(reader.statements()?.nested_block(block))?])
            }
            UnitOperation::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                let condition = reader.expr(condition)?;
                let mut statements = std::mem::take(&mut reader.prelude);
                let then_block = reader.effect_branch(then_value, input.scope())?;
                let else_block = match else_value {
                    Some(value) => reader.effect_branch(value, input.scope())?,
                    None => empty_branch(reader)?,
                };
                statements.push(c(reader
                    .statements()?
                    .if_statement(condition, then_block, else_block))?);
                Ok(statements)
            }
        };
        let statements = result?;
        #[cfg(unit_ast_probe)]
        ast::check(reader, &input, &statements);
        Ok(statements)
    }
}

fn empty_branch(reader: &mut Reader<'_>) -> Result<CBlock> {
    let parent = reader
        .active_scope
        .clone()
        .ok_or("missing unit branch scope")?;
    let index = reader.next_temporary;
    reader.next_temporary = index.checked_add(1).ok_or("unit branch budget exceeded")?;
    let key = CDeclarationKey {
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::EvaluationTemporary),
        name: c(CIdentifier::new(&format!("unit_else{index}")))?,
    };
    let scope = c(reader
        .registry
        .register_scope(&reader.function, Some(&parent), key))?;
    c(reader.statements()?.block(scope, vec![]))
}
