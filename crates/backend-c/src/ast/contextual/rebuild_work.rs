//! Postorder structural reconstruction without one host stack frame per block.
use super::{CBlock, CFunctionRef, CStatement, CStatementKind as K, CStatements, E, Recheck};
use std::collections::VecDeque;

pub(super) enum Built {
    Block(Box<CBlock>),
    Statement(Box<CStatement>),
}

enum Work<'a> {
    Block(&'a CBlock),
    Statement(&'a CStatement),
    FinishBlock(&'a CBlock),
    FinishStatement(&'a CStatement, usize),
}

pub(super) fn block(children: &mut VecDeque<Built>) -> Result<CBlock, E> {
    match children.pop_front() {
        Some(Built::Block(block)) => Ok(*block),
        _ => Err(E::StoredStructureMismatch),
    }
}

pub(super) fn statement(children: &mut VecDeque<Built>) -> Result<CStatement, E> {
    match children.pop_front() {
        Some(Built::Statement(statement)) => Ok(*statement),
        _ => Err(E::StoredStructureMismatch),
    }
}

fn children(statement: &CStatement) -> Vec<Work<'_>> {
    match statement.kind() {
        K::Block(block) => vec![Work::Block(block)],
        K::If {
            then_block,
            else_block,
            ..
        } => vec![Work::Block(then_block), Work::Block(else_block)],
        K::BoundedLoop { body, .. } => vec![Work::Block(body)],
        K::Switch { arms, default, .. } => arms
            .iter()
            .map(|arm| Work::Block(arm.body()))
            .chain(std::iter::once(Work::Block(default)))
            .collect(),
        K::Label { statement, .. } => vec![Work::Statement(statement)],
        K::Empty
        | K::Declare(_)
        | K::Assign { .. }
        | K::Evaluate(_)
        | K::Discard(_)
        | K::Break(_)
        | K::Continue(_)
        | K::Return(_)
        | K::CleanupJump(_) => vec![],
    }
}

pub(super) fn rebuild(
    checker: &Recheck<'_>,
    function: &CFunctionRef,
    root: &CBlock,
) -> Result<CBlock, E> {
    let mut pending = vec![Work::Block(root)];
    let mut built = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Block(block) => {
                pending.push(Work::FinishBlock(block));
                pending.extend(block.statements().iter().rev().map(Work::Statement));
            }
            Work::Statement(statement) => {
                let descendants = children(statement);
                pending.push(Work::FinishStatement(statement, descendants.len()));
                pending.extend(descendants.into_iter().rev());
            }
            Work::FinishBlock(block) => {
                let start = built
                    .len()
                    .checked_sub(block.statements().len())
                    .ok_or(E::StoredStructureMismatch)?;
                let statements = built
                    .split_off(start)
                    .into_iter()
                    .map(|value| match value {
                        Built::Statement(statement) => Ok(*statement),
                        Built::Block(_) => Err(E::StoredStructureMismatch),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let ast = CStatements::new(checker.registry, function.clone())?;
                built.push(Built::Block(Box::new(
                    ast.block(block.scope().clone(), statements)?,
                )));
            }
            Work::FinishStatement(statement, count) => {
                let start = built
                    .len()
                    .checked_sub(count)
                    .ok_or(E::StoredStructureMismatch)?;
                let mut children = VecDeque::from(built.split_off(start));
                let value = checker.statement(function, statement, &mut children)?;
                if !children.is_empty() {
                    return Err(E::StoredStructureMismatch);
                }
                built.push(Built::Statement(Box::new(value)));
            }
        }
    }
    let mut built = VecDeque::from(built);
    let result = block(&mut built)?;
    if !built.is_empty() {
        return Err(E::StoredStructureMismatch);
    }
    Ok(result)
}
