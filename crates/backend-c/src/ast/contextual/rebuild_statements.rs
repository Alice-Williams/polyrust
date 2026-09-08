//! Recheck actual statements, not just their cached containing function.

use super::super::{CBlock, CFunctionRef, CStatement, CStatementKind, CStatements};
use super::{CContextError as E, Recheck};

impl Recheck<'_> {
    pub(super) fn block(&self, function: &CFunctionRef, block: &CBlock) -> Result<CBlock, E> {
        let ast = CStatements::new(self.registry, function.clone())?;
        let statements = block
            .statements()
            .iter()
            .map(|value| self.statement(function, value))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ast.block(block.scope().clone(), statements)?)
    }

    fn statement(&self, function: &CFunctionRef, statement: &CStatement) -> Result<CStatement, E> {
        if statement.function() != function {
            return Err(E::StoredStructureMismatch);
        }
        let ast = CStatements::new(self.registry, function.clone())?;
        Ok(match statement.kind() {
            CStatementKind::Empty => ast.empty(),
            CStatementKind::Block(block) => ast.nested_block(self.block(function, block)?)?,
            CStatementKind::Declare(value) => ast.declare(
                value.local().clone(),
                value
                    .initializer()
                    .map(|value| self.initializer(value))
                    .transpose()?,
            )?,
            CStatementKind::Assign { place, value } => {
                ast.assign(self.place(place)?, self.value(value)?)?
            }
            CStatementKind::Evaluate(effect) => ast.evaluate(self.effect(effect)?)?,
            CStatementKind::Discard(value) => ast.discard(self.value(value)?)?,
            CStatementKind::If {
                condition,
                then_block,
                else_block,
            } => ast.if_statement(
                self.value(condition)?,
                self.block(function, then_block)?,
                self.block(function, else_block)?,
            )?,
            CStatementKind::BoundedLoop {
                identity,
                progress,
                condition,
                body,
            } => ast.counted_loop(
                identity.clone(),
                progress.counter().clone(),
                progress.bound().clone(),
                self.value(condition)?,
                self.block(function, body)?,
            )?,
            CStatementKind::Switch {
                identity,
                value,
                arms,
                default,
            } => {
                let arms = arms
                    .iter()
                    .map(|arm| {
                        Ok(ast
                            .switch_arm(arm.cases().to_vec(), self.block(function, arm.body())?)?)
                    })
                    .collect::<Result<Vec<_>, E>>()?;
                ast.switch_statement(
                    identity.clone(),
                    self.value(value)?,
                    arms,
                    self.block(function, default)?,
                )?
            }
            CStatementKind::Break(target) => ast.break_statement(target.clone())?,
            CStatementKind::Continue(identity) => ast.continue_statement(identity.clone())?,
            CStatementKind::Return(value) => {
                ast.return_statement(value.as_ref().map(|value| self.value(value)).transpose()?)?
            }
            CStatementKind::CleanupJump(identity) => ast.cleanup_jump(identity.clone())?,
            CStatementKind::Label {
                identity,
                statement,
            } => ast.label(identity.clone(), self.statement(function, statement)?)?,
        })
    }
}
