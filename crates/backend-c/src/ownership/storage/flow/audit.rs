//! All reachable actions/edges precede secondary exit obligations.
use super::{E, product::Product};
use crate::ast::contextual::flow_graph::{Action, Destination, Graph};
use crate::ownership::{context_facts::ContextFacts, loops::LoopEvidence, numeric_flow};
use std::collections::{BTreeSet, VecDeque};

pub(super) fn check<'ast>(
    context: &ContextFacts<'ast>,
    graph: &Graph<'ast>,
    incoming: &[Option<Product<'ast>>],
    loops: &[LoopEvidence<'ast>],
) -> Result<(), E> {
    let mut pending = VecDeque::from([graph.entry()]);
    let mut seen = BTreeSet::new();
    let mut exits = vec![];
    while let Some(point) = pending.pop_front() {
        if !seen.insert(point) {
            continue;
        }
        let Some(mut state) = incoming[point.index()].clone() else {
            continue;
        };
        if !numeric_flow::progress(&mut state.numeric, graph, point, loops)? {
            continue;
        }
        let node = graph.node(point);
        state.action(context, graph, point, true)?;
        if matches!(node.action(), Action::FunctionEnd)
            || node
                .successors()
                .iter()
                .any(|edge| edge.destination() == Destination::FunctionReturn)
        {
            exits.push(state.memory.allocations.clone());
        }
        for edge in node.successors() {
            if let Destination::Point(next) = edge.destination()
                && state.edge(context, graph, point, edge, true)?.is_some()
            {
                pending.push_back(next);
            }
        }
    }
    for allocations in exits {
        allocations.finish()?;
    }
    Ok(())
}
