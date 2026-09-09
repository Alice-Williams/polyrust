//! Structural construction retains actual predicates, cases and control targets.
use super::{
    Action, BranchOwner, Destination, Edge, EdgeMeaning, Graph, Node, Point, Polarity, Selection,
    scope_edges,
};
use crate::ast::{
    CBlock, CBreakTarget, CCleanupExitRef, CContextError as E, CScopeRef, CStatement,
    CStatementKind,
};
use std::collections::BTreeMap;

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

pub(super) fn build(body: &CBlock) -> Result<Graph<'_>, E> {
    let mut builder = Builder {
        nodes: Vec::new(),
        controls: Vec::new(),
        labels: BTreeMap::new(),
    };
    let end = builder.node(body.scope(), Action::FunctionEnd, vec![]);
    let entry = builder.block(body, end, EdgeMeaning::Flow)?;
    for node in &mut builder.nodes {
        if let Action::CleanupJump(identity) = node.action {
            let target = *builder.labels.get(identity).ok_or(E::InvalidCleanupExit)?;
            node.successors
                .push(edge(target, EdgeMeaning::Cleanup(identity)));
        }
    }
    let mut graph = Graph {
        nodes: builder.nodes,
        entry,
        function: body.scope().function(),
    };
    scope_edges::derive(&mut graph)?;
    Ok(graph)
}

fn edge(destination: Point, meaning: EdgeMeaning<'_>) -> Edge<'_> {
    Edge {
        destination: Destination::Point(destination),
        meaning,
        exited_scopes: vec![],
    }
}

impl<'a> Builder<'a> {
    fn node(
        &mut self,
        scope: &'a CScopeRef,
        action: Action<'a>,
        successors: Vec<Edge<'a>>,
    ) -> Point {
        let point = Point(self.nodes.len());
        self.nodes.push(Node {
            origin: None,
            action,
            scope,
            successors,
        });
        point
    }

    fn block(&mut self, block: &'a CBlock, next: Point, exit: EdgeMeaning<'a>) -> Result<Point, E> {
        let mut next = self.node(
            block.scope(),
            Action::ScopeExit(block.scope()),
            vec![edge(next, exit)],
        );
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
            CStatementKind::Discard(value) => Action::Discard(value),
            CStatementKind::Block(block) => return self.block(block, next, EdgeMeaning::Flow),
            CStatementKind::Return(value) => {
                return Ok(self.node(
                    scope,
                    Action::Return(value.as_ref()),
                    vec![Edge {
                        destination: Destination::FunctionReturn,
                        meaning: EdgeMeaning::Return,
                        exited_scopes: vec![],
                    }],
                ));
            }
            CStatementKind::CleanupJump(value) => {
                return Ok(self.node(scope, Action::CleanupJump(value), vec![]));
            }
            CStatementKind::Label {
                identity,
                statement,
            } => {
                let child = self.statement(scope, statement, next)?;
                let point = self.node(
                    scope,
                    Action::Label(identity),
                    vec![edge(child, EdgeMeaning::Flow)],
                );
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
                let then_entry = self.block(then_block, next, EdgeMeaning::Flow)?;
                let else_entry = self.block(else_block, next, EdgeMeaning::Flow)?;
                return Ok(self.node(
                    scope,
                    Action::Read(condition),
                    vec![
                        edge(
                            then_entry,
                            EdgeMeaning::Predicate {
                                condition,
                                polarity: Polarity::True,
                                owner: BranchOwner::If,
                            },
                        ),
                        edge(
                            else_entry,
                            EdgeMeaning::Predicate {
                                condition,
                                polarity: Polarity::False,
                                owner: BranchOwner::If,
                            },
                        ),
                    ],
                ));
            }
            CStatementKind::BoundedLoop {
                identity,
                condition,
                body,
                ..
            } => {
                let test = self.node(
                    scope,
                    Action::Read(condition),
                    vec![edge(
                        next,
                        EdgeMeaning::Predicate {
                            condition,
                            polarity: Polarity::False,
                            owner: BranchOwner::Loop(identity),
                        },
                    )],
                );
                self.controls.push(Control {
                    target: CBreakTarget::Loop(identity.clone()),
                    exit: next,
                    repeat: Some(test),
                });
                let body_entry = self.block(body, test, EdgeMeaning::Backedge(identity))?;
                self.controls.pop();
                self.nodes[test.0].successors.push(edge(
                    body_entry,
                    EdgeMeaning::Predicate {
                        condition,
                        polarity: Polarity::True,
                        owner: BranchOwner::Loop(identity),
                    },
                ));
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
                    let entry = self.block(arm.body(), end, EdgeMeaning::Flow)?;
                    entries.push(edge(
                        entry,
                        EdgeMeaning::Switch {
                            identity,
                            value,
                            selection: Selection::Cases(arm.cases()),
                        },
                    ));
                }
                let entry = self.block(default, end, EdgeMeaning::Flow)?;
                entries.push(edge(
                    entry,
                    EdgeMeaning::Switch {
                        identity,
                        value,
                        selection: Selection::Default,
                    },
                ));
                self.controls.pop();
                return Ok(self.node(scope, Action::Read(value), entries));
            }
            CStatementKind::Break(target) => {
                let control = self.controls.last().ok_or(E::WrongControlTarget)?;
                if &control.target != target {
                    return Err(E::WrongControlTarget);
                }
                return Ok(self.node(
                    scope,
                    Action::Empty,
                    vec![edge(control.exit, EdgeMeaning::Break(target))],
                ));
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
                let repeat = control.repeat.ok_or(E::WrongControlTarget)?;
                return Ok(self.node(
                    scope,
                    Action::Empty,
                    vec![edge(repeat, EdgeMeaning::Continue(identity))],
                ));
            }
        };
        Ok(self.node(scope, action, vec![edge(next, EdgeMeaning::Flow)]))
    }
}
