//! Diagnose actual actions before secondary obligations from poisoned exits.
use super::{E, Engine, NumericFacts, State};
use crate::ast::contextual::flow_graph::{Action, Destination, Graph};
use std::collections::{BTreeSet, VecDeque};

pub(super) fn check<'ast>(
    facts: &NumericFacts<'ast>,
    graph: &Graph<'ast>,
    incoming: &[Option<State>],
) -> Result<(), E> {
    let mut pending = VecDeque::from([graph.entry()]);
    let mut seen = BTreeSet::new();
    let mut exits = vec![];
    while let Some(point) = pending.pop_front() {
        if !seen.insert(point) || !facts.reachable(graph, point) {
            continue;
        }
        let Some(mut state) = incoming[point.index()].clone() else {
            continue;
        };
        let mut engine = Engine {
            facts,
            cursor: Some(facts.cursor(graph, point)?),
        };
        let node = graph.node(point);
        engine.action(node.action(), &mut state)?;
        // A void fallthrough has no successor. Return edges have an explicit
        // destination. Both are checked only after every actual action succeeds.
        if matches!(node.action(), Action::FunctionEnd)
            || node
                .successors()
                .iter()
                .any(|edge| edge.destination() == Destination::FunctionReturn)
        {
            exits.push(state.allocations);
        }
        for edge in node.successors() {
            if let Destination::Point(next) = edge.destination() {
                pending.push_back(next);
            }
        }
    }
    for allocations in exits {
        allocations.finish()?;
    }
    Ok(())
}
