//! Closed block endings keep source return structure distinct from tail values.
#[path = "exit_routes/early.rs"]
pub(super) mod early;
use super::{LinearError as Error, Result};
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Tail,
    Return,
    Guarded(Outcome),
    Early(Outcome),
    Selection(Outcome),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    False,
    True,
}
impl Outcome {
    pub(super) fn value(self) -> u128 {
        match self {
            Self::False => 0,
            Self::True => 1,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Exit<'tcx> {
    Tail(&'tcx hir::Expr<'tcx>),
    Return {
        expression: &'tcx hir::Expr<'tcx>,
        value: &'tcx hir::Expr<'tcx>,
    },
}
impl<'tcx> Exit<'tcx> {
    pub(super) fn value(self) -> &'tcx hir::Expr<'tcx> {
        match self {
            Self::Tail(value) | Self::Return { value, .. } => value,
        }
    }
    pub(super) fn mode(self) -> Mode {
        match self {
            Self::Tail(_) => Mode::Tail,
            Self::Return { .. } => Mode::Return,
        }
    }
    pub(super) fn same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Tail(a), Self::Tail(b)) => std::ptr::eq(a, b),
            (
                Self::Return {
                    expression: a,
                    value: av,
                },
                Self::Return {
                    expression: b,
                    value: bv,
                },
            ) => std::ptr::eq(a, b) && std::ptr::eq(av, bv),
            _ => false,
        }
    }
}

pub(super) enum End<'tcx> {
    Nested(&'tcx hir::Block<'tcx>),
    Exit(Exit<'tcx>),
}

pub(super) fn parts<'tcx>(
    block: &'tcx hir::Block<'tcx>,
    mode: Mode,
) -> Result<(&'tcx [hir::Stmt<'tcx>], End<'tcx>)> {
    if let Mode::Early(outcome) = mode {
        return early::select(block, outcome);
    }
    let (statements, expression) = match (block.expr, mode) {
        (Some(expression), _) => (block.stmts, expression),
        (None, Mode::Return | Mode::Guarded(_) | Mode::Selection(_)) => {
            let (last, prefix) = block.stmts.split_last().ok_or(Error::BodyShape)?;
            let hir::StmtKind::Semi(expression) = last.kind else {
                return Err(Error::BodyShape);
            };
            (prefix, expression)
        }
        _ => return Err(Error::BodyShape),
    };
    let end = match (expression.kind, mode) {
        (hir::ExprKind::If(_, yes, Some(no)), Mode::Guarded(outcome)) => {
            let arm = match outcome {
                Outcome::False => no,
                Outcome::True => yes,
            };
            let hir::ExprKind::Block(child, None) = arm.kind else {
                return Err(Error::BodyShape);
            };
            End::Nested(child)
        }
        (hir::ExprKind::Block(child, None), _) if block.expr.is_some() => End::Nested(child),
        (hir::ExprKind::Ret(Some(value)), Mode::Return | Mode::Guarded(_) | Mode::Selection(_)) => {
            End::Exit(Exit::Return { expression, value })
        }
        (_, Mode::Tail | Mode::Selection(_)) => End::Exit(Exit::Tail(expression)),
        _ => return Err(Error::BodyShape),
    };
    Ok((statements, end))
}
