//! Immediate parent/owner checks; occurrence and dominance proof belongs to 02C.

use super::{
    CBlock, CScopeRef, CStatement, CStatementError as E, CStatementKind as K, CStatements,
};

impl CStatements<'_> {
    pub(super) fn check_placement(
        &self,
        scope: &CScopeRef,
        mut statement: &CStatement,
    ) -> Result<(), E> {
        // Labels retain their containing scope, so walk their chain without
        // consuming a host stack frame for each nested label.
        while let K::Label {
            identity,
            statement: child,
        } = statement.kind()
        {
            same_scope(scope, identity.scope())?;
            statement = child;
        }
        match statement.kind() {
            K::Declare(value) => same_scope(scope, value.local().scope()),
            K::Block(block) => child_scope(scope, block),
            K::If {
                then_block,
                else_block,
                ..
            } => {
                child_scope(scope, then_block)?;
                child_scope(scope, else_block)
            }
            K::BoundedLoop { identity, body, .. } => {
                same_scope(scope, identity.scope())?;
                child_scope(scope, body)
            }
            K::Switch {
                identity,
                arms,
                default,
                ..
            } => {
                same_scope(scope, identity.scope())?;
                child_scope(scope, default)?;
                for arm in arms {
                    child_scope(scope, arm.body())?;
                }
                Ok(())
            }
            K::Label { .. } => unreachable!("label chain consumed above"),
            K::Empty
            | K::Assign { .. }
            | K::Evaluate(_)
            | K::Discard(_)
            | K::Break(_)
            | K::Continue(_)
            | K::Return(_)
            | K::CleanupJump(_) => Ok(()),
        }
    }
}

fn same_scope(left: &CScopeRef, right: &CScopeRef) -> Result<(), E> {
    if left == right {
        Ok(())
    } else {
        Err(E::WrongScope)
    }
}

fn child_scope(parent: &CScopeRef, child: &CBlock) -> Result<(), E> {
    if child.scope().parent() == Some(parent) {
        Ok(())
    } else {
        Err(E::WrongScope)
    }
}
