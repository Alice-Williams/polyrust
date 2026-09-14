//! Recheck actual statements, not just their cached containing function.

use super::super::{CBlock, CFunctionRef, CStatement, CStatementKind, CStatements};
use super::{CContextError as E, Recheck};
use std::collections::VecDeque;

#[path = "rebuild_work.rs"]
mod work;
use work::Built;

impl Recheck<'_> {
    pub(super) fn block(&self, function: &CFunctionRef, block: &CBlock) -> Result<CBlock, E> {
        work::rebuild(self, function, block)
    }

    fn statement(
        &self,
        function: &CFunctionRef,
        statement: &CStatement,
        children: &mut VecDeque<Built>,
    ) -> Result<CStatement, E> {
        if statement.function() != function {
            return Err(E::StoredStructureMismatch);
        }
        let ast = CStatements::new(self.registry, function.clone())?;
        Ok(match statement.kind() {
            CStatementKind::Empty => ast.empty(),
            CStatementKind::Block(_) => ast.nested_block(work::block(children)?)?,
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
            CStatementKind::If { condition, .. } => ast.if_statement(
                self.value(condition)?,
                work::block(children)?,
                work::block(children)?,
            )?,
            CStatementKind::BoundedLoop {
                identity,
                progress,
                condition,
                ..
            } => ast.counted_loop(
                identity.clone(),
                progress.counter().clone(),
                progress.bound().clone(),
                self.value(condition)?,
                work::block(children)?,
            )?,
            CStatementKind::Switch {
                identity,
                value,
                arms,
                ..
            } => {
                let arms = arms
                    .iter()
                    .map(|arm| Ok(ast.switch_arm(arm.cases().to_vec(), work::block(children)?)?))
                    .collect::<Result<Vec<_>, E>>()?;
                ast.switch_statement(
                    identity.clone(),
                    self.value(value)?,
                    arms,
                    work::block(children)?,
                )?
            }
            CStatementKind::Break(target) => ast.break_statement(target.clone())?,
            CStatementKind::Continue(identity) => ast.continue_statement(identity.clone())?,
            CStatementKind::Return(value) => {
                ast.return_statement(value.as_ref().map(|value| self.value(value)).transpose()?)?
            }
            CStatementKind::CleanupJump(identity) => ast.cleanup_jump(identity.clone())?,
            CStatementKind::Label { identity, .. } => {
                ast.label(identity.clone(), work::statement(children)?)?
            }
        })
    }
}
