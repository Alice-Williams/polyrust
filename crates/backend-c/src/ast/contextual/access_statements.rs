//! All-syntax object accesses, including unreachable branches and definitions.

use super::super::{CBlock, CDefinitionKind, CFileItem, CSourceFile, CStatement, CStatementKind};
use super::access_walk::{self as walk, Access, Visitor};

pub(crate) fn file<V: Visitor>(visitor: &mut V, file: &CSourceFile) -> Result<(), V::Error> {
    for item in file.items() {
        match item {
            CFileItem::Definition(value) => match value.kind() {
                CDefinitionKind::Function { body, .. } => block(visitor, body)?,
                CDefinitionKind::Object { initializer, .. } => {
                    walk::initializer(visitor, initializer)?
                }
            },
            CFileItem::StaticAssert(value) => walk::expression(visitor, value.condition())?,
            CFileItem::Declaration(_) | CFileItem::Comment(_) => {}
        }
    }
    Ok(())
}

fn block<V: Visitor>(visitor: &mut V, value: &CBlock) -> Result<(), V::Error> {
    for value in value.statements() {
        statement(visitor, value)?;
    }
    Ok(())
}

fn statement<V: Visitor>(visitor: &mut V, value: &CStatement) -> Result<(), V::Error> {
    match value.kind() {
        CStatementKind::Block(value) => block(visitor, value)?,
        CStatementKind::Declare(value) => {
            if let Some(value) = value.initializer() {
                walk::initializer(visitor, value)?;
            }
        }
        CStatementKind::Assign { place, value } => {
            walk::place(visitor, place, Access::Write)?;
            walk::expression(visitor, value)?;
        }
        CStatementKind::Evaluate(value) => walk::call(visitor, value.call())?,
        CStatementKind::Discard(value) => walk::expression(visitor, value)?,
        CStatementKind::If {
            condition,
            then_block,
            else_block,
        } => {
            walk::expression(visitor, condition)?;
            block(visitor, then_block)?;
            block(visitor, else_block)?;
        }
        CStatementKind::BoundedLoop {
            condition, body, ..
        } => {
            walk::expression(visitor, condition)?;
            block(visitor, body)?;
        }
        CStatementKind::Switch {
            value,
            arms,
            default,
            ..
        } => {
            walk::expression(visitor, value)?;
            for arm in arms {
                block(visitor, arm.body())?;
            }
            block(visitor, default)?;
        }
        CStatementKind::Return(value) => {
            if let Some(value) = value {
                walk::expression(visitor, value)?;
            }
        }
        CStatementKind::Label {
            statement: value, ..
        } => statement(visitor, value)?,
        CStatementKind::Empty
        | CStatementKind::Break(_)
        | CStatementKind::Continue(_)
        | CStatementKind::CleanupJump(_) => {}
    }
    Ok(())
}
