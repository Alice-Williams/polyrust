//! Actual-AST progress evidence; no public proof constructors or certificates.
mod inventory;
mod paths;
mod shape;

use super::{CSafetyError as E, context_facts::ContextFacts};
use crate::ast::{
    CBlock, CCountedProgress, CLocalDeclaration, CLoopRef, CRegistry, CSourceFile, CStatement,
    CStatementKind, CValue, contextual::flow_graph::Graph,
};

pub(super) struct LoopEvidence<'a> {
    statement: &'a CStatement,
    identity: &'a CLoopRef,
    progress: &'a CCountedProgress,
    condition: &'a CValue,
    body: &'a CBlock,
    counter: &'a CLocalDeclaration,
    bound: &'a CLocalDeclaration,
    steps: Vec<&'a CStatement>,
}

pub(super) fn check<'a>(context: &ContextFacts<'a>) -> Result<Vec<LoopEvidence<'a>>, E> {
    let mut checked = Vec::new();
    for graph in context.functions() {
        let mut loops = candidates(graph)?;
        inventory::check(context, graph, &mut loops)?;
        for evidence in loops {
            paths::check(context.registry(), graph, &evidence)?;
            checked.push(evidence);
        }
    }
    Ok(checked)
}

fn candidates<'a>(graph: &Graph<'a>) -> Result<Vec<LoopEvidence<'a>>, E> {
    let mut result = Vec::new();
    for node in graph.nodes() {
        let Some(statement) = node.origin() else {
            continue;
        };
        if let CStatementKind::BoundedLoop {
            identity,
            progress,
            condition,
            body,
        } = statement.kind()
        {
            let candidate = LoopEvidence {
                statement,
                identity,
                progress,
                condition,
                body,
                counter: shape::declaration(graph, progress.counter())?,
                bound: shape::declaration(graph, progress.bound())?,
                steps: Vec::new(),
            };
            shape::check(&candidate)?;
            result.push(candidate);
        }
    }
    Ok(result)
}

impl CRegistry {
    /// Checks exact counted-loop structure and once-per-continuing-path updates.
    /// Numeric flow, ownership, call effects and rendering remain unproved.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::loops::LoopEvidence;
    /// fn forge<'a>() -> LoopEvidence<'a> { todo!() }
    /// ```
    pub fn check_counted_loops(&self, files: &[CSourceFile]) -> Result<(), E> {
        let context = ContextFacts::check(self, files)?;
        check(&context)?;
        Ok(())
    }
}
