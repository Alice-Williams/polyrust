//! Immutable graph derived from actual control nodes, never input-authored IDs.
mod builder;
mod scope_edges;

#[cfg(test)]
#[path = "../../tests/control_graph_edges.rs"]
mod edge_tests;
#[cfg(test)]
#[path = "../../tests/control_graph_exits.rs"]
mod exit_tests;
#[cfg(test)]
#[path = "../../tests/control_graph_loops.rs"]
mod loop_tests;

use super::CContextError;
use crate::ast::{
    CBlock, CBreakTarget, CCaseConstant, CCleanupExitRef, CEffect, CFunctionRef, CLocalDeclaration,
    CLoopRef, CPlace, CScopeRef, CStatement, CSwitchRef, CValue,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Point(usize);
impl Point {
    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

pub(crate) enum Action<'a> {
    Declare(&'a CLocalDeclaration),
    Assign(&'a CPlace, &'a CValue),
    Evaluate(&'a CEffect),
    Read(&'a CValue),
    Discard(&'a CValue),
    ScopeExit(&'a CScopeRef),
    Return(Option<&'a CValue>),
    CleanupJump(&'a CCleanupExitRef),
    Label(&'a CCleanupExitRef),
    FunctionEnd,
    CaseEnd,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Destination {
    Point(Point),
    FunctionReturn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Polarity {
    True,
    False,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BranchOwner<'a> {
    If,
    Loop(&'a CLoopRef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Selection<'a> {
    Cases(&'a [CCaseConstant]),
    Default,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EdgeMeaning<'a> {
    Flow,
    Predicate {
        condition: &'a CValue,
        polarity: Polarity,
        owner: BranchOwner<'a>,
    },
    Switch {
        identity: &'a CSwitchRef,
        value: &'a CValue,
        selection: Selection<'a>,
    },
    Backedge(&'a CLoopRef),
    Break(&'a CBreakTarget),
    Continue(&'a CLoopRef),
    Return,
    Cleanup(&'a CCleanupExitRef),
}

pub(crate) struct Edge<'a> {
    destination: Destination,
    meaning: EdgeMeaning<'a>,
    exited_scopes: Vec<&'a CScopeRef>,
}
impl<'a> Edge<'a> {
    pub(crate) const fn destination(&self) -> Destination {
        self.destination
    }
    pub(crate) const fn meaning(&self) -> EdgeMeaning<'a> {
        self.meaning
    }
    pub(crate) fn exited_scopes(&self) -> &[&'a CScopeRef] {
        &self.exited_scopes
    }
}

pub(crate) struct Node<'a> {
    origin: Option<&'a CStatement>,
    action: Action<'a>,
    scope: &'a CScopeRef,
    successors: Vec<Edge<'a>>,
}
impl<'a> Node<'a> {
    pub(crate) const fn origin(&self) -> Option<&'a CStatement> {
        self.origin
    }
    pub(crate) const fn action(&self) -> &Action<'a> {
        &self.action
    }
    pub(crate) const fn scope(&self) -> &'a CScopeRef {
        self.scope
    }
    pub(crate) fn successors(&self) -> &[Edge<'a>] {
        &self.successors
    }
}

pub(crate) struct Graph<'a> {
    nodes: Vec<Node<'a>>,
    entry: Point,
    function: &'a CFunctionRef,
}
impl<'a> Graph<'a> {
    pub(crate) fn build(body: &'a CBlock) -> Result<Self, CContextError> {
        builder::build(body)
    }
    pub(crate) fn nodes(&self) -> &[Node<'a>] {
        &self.nodes
    }
    pub(crate) const fn entry(&self) -> Point {
        self.entry
    }
    pub(crate) const fn function(&self) -> &'a CFunctionRef {
        self.function
    }
    pub(crate) fn node(&self, point: Point) -> &Node<'a> {
        &self.nodes[point.0]
    }
}
