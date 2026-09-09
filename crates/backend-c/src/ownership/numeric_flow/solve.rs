//! Worklist convergence precedes the strict safety pass; no timeout is success.
mod edges;
mod obligations;
mod progress;

use super::{Analysis, E, Engine, FunctionFacts, Mode, Site, State, storage};
use crate::ast::contextual::flow_graph::{Destination, EdgeMeaning, Graph};
use crate::ownership::{context_facts::ContextFacts, layout::Layouts, loops};
use std::collections::VecDeque;

pub(super) fn check<'a>(context: &ContextFacts<'a>) -> Result<Analysis<'a>, E> {
    let loops = loops::check(context)?;
    let mut engine = Engine {
        registry: context.registry(),
        layouts: Layouts::new(context.registry()),
        addresses: storage::addresses(context)?,
        mode: Mode::Solve,
        obligations: vec![],
    };
    let mut functions = Vec::new();
    for graph in context.functions() {
        engine.mode = Mode::Solve;
        let incoming = fixed_point(&mut engine, graph, &loops)?;
        let mut verified = vec![None; graph.nodes().len()];
        for (point, state) in graph.points().zip(incoming) {
            let Some(mut state) = state else { continue };
            if !progress::refine(&mut state, graph, point, &loops)? {
                continue;
            }
            engine.mode = Mode::Verify(Site {
                function: graph.function(),
                point,
            });
            verified[point.index()] = Some(state.clone());
            engine.action(graph.node(point).action(), &mut state)?;
        }
        functions.push(FunctionFacts {
            function: graph.function(),
            incoming: verified,
        });
    }
    engine.check_obligations()?;
    Ok(Analysis {
        functions,
        obligations: engine.obligations,
    })
}

fn fixed_point<'a>(
    engine: &mut Engine<'a>,
    graph: &Graph<'a>,
    loops: &[loops::LoopEvidence<'a>],
) -> Result<Vec<Option<State<'a>>>, E> {
    let mut incoming = vec![None; graph.nodes().len()];
    incoming[graph.entry().index()] = Some(State::default());
    let mut pending = VecDeque::from([graph.entry()]);
    while let Some(point) = pending.pop_front() {
        let Some(mut state) = incoming[point.index()].clone() else {
            continue;
        };
        if !progress::refine(&mut state, graph, point, loops)? {
            continue;
        }
        engine.action(graph.node(point).action(), &mut state)?;
        for edge in graph.node(point).successors() {
            let Destination::Point(next) = edge.destination() else {
                continue;
            };
            let Some(mut outgoing) = engine.edge(&state, graph.node(point), edge)? else {
                continue;
            };
            for scope in edge.exited_scopes() {
                outgoing.leave(scope);
            }
            if !progress::refine(&mut outgoing, graph, next, loops)? {
                continue;
            }
            let widen = matches!(
                edge.meaning(),
                EdgeMeaning::Backedge(_) | EdgeMeaning::Continue(_)
            );
            let joined = match &incoming[next.index()] {
                Some(old) => old.join(&outgoing, widen)?,
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
