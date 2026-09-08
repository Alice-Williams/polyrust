//! Private graph derived from actual control nodes, never input-authored IDs.

use super::super::{
    CBlock, CBreakTarget, CCleanupExitRef, CEffect, CLocalDeclaration, CPlace, CScopeRef,
    CStatement, CStatementKind, CValue,
};
use super::CContextError as E;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Point(pub(super) usize);

pub(super) enum Action<'a> {
    Declare(&'a CLocalDeclaration),
    Assign(&'a CPlace, &'a CValue),
    Evaluate(&'a CEffect),
    Read(&'a CValue),
    ScopeExit(&'a CScopeRef),
    Return(Option<&'a CValue>),
    CleanupJump(&'a CCleanupExitRef),
    Label(&'a CCleanupExitRef),
    FunctionEnd,
    CaseEnd,
    Empty,
}

pub(super) struct Node<'a> {
    pub(super) origin: Option<&'a CStatement>,
    pub(super) action: Action<'a>,
    pub(super) scope: &'a CScopeRef,
    pub(super) successors: Vec<Point>,
}

pub(super) struct Graph<'a> {
    pub(super) nodes: Vec<Node<'a>>,
    pub(super) entry: Point,
}

struct Control {
    target: CBreakTarget,
    exit: Point,
    repeat: Option<Point>,
}

struct Builder<'a> {
    nodes: Vec<Node<'a>>,
    controls: Vec<Control>,
    labels: BTreeMap<&'a CCleanupExitRef, Point>,
}

impl<'a> Graph<'a> {
    pub(super) fn build(body: &'a CBlock) -> Result<Self, E> {
        let mut builder = Builder {
            nodes: Vec::new(),
            controls: Vec::new(),
            labels: BTreeMap::new(),
        };
        let end = builder.node(body.scope(), Action::FunctionEnd, vec![]);
        let entry = builder.block(body, end)?;
        for node in &mut builder.nodes {
            if let Action::CleanupJump(identity) = node.action {
                node.successors
                    .push(*builder.labels.get(identity).ok_or(E::InvalidCleanupExit)?);
            }
        }
        Ok(Self {
            nodes: builder.nodes,
            entry,
        })
    }
}

impl<'a> Builder<'a> {
    fn node(&mut self, scope: &'a CScopeRef, action: Action<'a>, successors: Vec<Point>) -> Point {
        let point = Point(self.nodes.len());
        self.nodes.push(Node {
            origin: None,
            action,
            scope,
            successors,
        });
        point
    }

    fn block(&mut self, block: &'a CBlock, next: Point) -> Result<Point, E> {
        let mut next = self.node(block.scope(), Action::ScopeExit(block.scope()), vec![next]);
        for statement in block.statements().iter().rev() {
            next = self.statement(block.scope(), statement, next)?;
        }
        Ok(next)
    }

    fn statement(
        &mut self,
        scope: &'a CScopeRef,
        statement: &'a CStatement,
        next: Point,
    ) -> Result<Point, E> {
        let point = self.statement_entry(scope, statement, next)?;
        if !matches!(statement.kind(), CStatementKind::Block(_)) {
            // Keep the actual control/progress/transfer payload for the later
            // ownership and backedge proofs, not only an anonymous graph edge.
            self.nodes[point.0].origin = Some(statement);
        }
        Ok(point)
    }

    fn statement_entry(
        &mut self,
        scope: &'a CScopeRef,
        statement: &'a CStatement,
        next: Point,
    ) -> Result<Point, E> {
        let action = match statement.kind() {
            CStatementKind::Empty => Action::Empty,
            CStatementKind::Declare(value) => Action::Declare(value),
            CStatementKind::Assign { place, value } => Action::Assign(place, value),
            CStatementKind::Evaluate(value) => Action::Evaluate(value),
            CStatementKind::Discard(value) => Action::Read(value),
            CStatementKind::Block(block) => return self.block(block, next),
            CStatementKind::Return(value) => {
                return Ok(self.node(scope, Action::Return(value.as_ref()), vec![]));
            }
            CStatementKind::CleanupJump(value) => {
                return Ok(self.node(scope, Action::CleanupJump(value), vec![]));
            }
            CStatementKind::Label {
                identity,
                statement,
            } => {
                let child = self.statement(scope, statement, next)?;
                let point = self.node(scope, Action::Label(identity), vec![child]);
                if self.labels.insert(identity, point).is_some() {
                    return Err(E::DuplicateOccurrence);
                }
                return Ok(point);
            }
            CStatementKind::If {
                condition,
                then_block,
                else_block,
            } => {
                let then_entry = self.block(then_block, next)?;
                let else_entry = self.block(else_block, next)?;
                return Ok(self.node(scope, Action::Read(condition), vec![then_entry, else_entry]));
            }
            CStatementKind::BoundedLoop {
                identity,
                condition,
                body,
                ..
            } => {
                let test = self.node(scope, Action::Read(condition), vec![next]);
                self.controls.push(Control {
                    target: CBreakTarget::Loop(identity.clone()),
                    exit: next,
                    repeat: Some(test),
                });
                let body_entry = self.block(body, test)?;
                self.controls.pop();
                self.nodes[test.0].successors.push(body_entry);
                return Ok(test);
            }
            CStatementKind::Switch {
                identity,
                value,
                arms,
                default,
            } => {
                self.controls.push(Control {
                    target: CBreakTarget::Switch(identity.clone()),
                    exit: next,
                    repeat: None,
                });
                let end = self.node(scope, Action::CaseEnd, vec![]);
                let mut entries = Vec::new();
                for arm in arms {
                    entries.push(self.block(arm.body(), end)?);
                }
                entries.push(self.block(default, end)?);
                self.controls.pop();
                return Ok(self.node(scope, Action::Read(value), entries));
            }
            CStatementKind::Break(target) => {
                let control = self.controls.last().ok_or(E::WrongControlTarget)?;
                if &control.target != target {
                    return Err(E::WrongControlTarget);
                }
                return Ok(self.node(scope, Action::Empty, vec![control.exit]));
            }
            CStatementKind::Continue(identity) => {
                let control = self
                    .controls
                    .iter()
                    .rev()
                    .find(|value| value.repeat.is_some())
                    .ok_or(E::WrongControlTarget)?;
                if control.target != CBreakTarget::Loop(identity.clone()) {
                    return Err(E::WrongControlTarget);
                }
                return Ok(self.node(
                    scope,
                    Action::Empty,
                    vec![control.repeat.ok_or(E::WrongControlTarget)?],
                ));
            }
        };
        Ok(self.node(scope, action, vec![next]))
    }
}
