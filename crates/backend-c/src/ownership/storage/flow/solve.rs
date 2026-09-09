//! Widen only actual loop backedges; provisional failures are replayed strictly.
use super::{E, State, product::Product};
use crate::ast::contextual::flow_graph::{Destination, EdgeMeaning, Graph};
use crate::ownership::{context_facts::ContextFacts, loops::LoopEvidence, numeric_flow};
use std::collections::VecDeque;

pub(super) fn function<'ast>(
    context: &ContextFacts<'ast>,
    graph: &Graph<'ast>,
    entry: State,
    loops: &[LoopEvidence<'ast>],
) -> Result<Vec<Option<Product<'ast>>>, E> {
    let mut incoming = vec![None; graph.nodes().len()];
    incoming[graph.entry().index()] = Some(Product {
        memory: entry,
        numeric: numeric_flow::State::default(),
    });
    let mut pending = VecDeque::from([graph.entry()]);
    while let Some(point) = pending.pop_front() {
        let Some(mut state) = incoming[point.index()].clone() else {
            continue;
        };
        if !numeric_flow::progress(&mut state.numeric, graph, point, loops)? {
            continue;
        }
        state.action(context, graph, point, loops, false)?;
        for edge in graph.node(point).successors() {
            let Destination::Point(next) = edge.destination() else {
                continue;
            };
            let Some(mut outgoing) = state.edge(context, graph, point, edge, loops, false)? else {
                continue;
            };
            if !numeric_flow::progress(&mut outgoing.numeric, graph, next, loops)? {
                continue;
            }
            let widen = matches!(
                edge.meaning(),
                EdgeMeaning::Backedge(_) | EdgeMeaning::Continue(_)
            );
            let joined = match &incoming[next.index()] {
                Some(old) => Product {
                    memory: old.memory.join(&outgoing.memory, context.registry())?,
                    numeric: old.numeric.join(&outgoing.numeric, widen)?,
                },
                None => outgoing,
            };
            if incoming[next.index()].as_ref() != Some(&joined) {
                incoming[next.index()] = Some(joined);
                pending.push_back(next);
            }
        }
    }
    Ok(incoming)
}
