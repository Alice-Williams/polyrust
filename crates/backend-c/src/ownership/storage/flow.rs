//! Finite must-flow over authenticated points; strict checks follow convergence.
mod audit;
use super::{Engine, state::State, values::Cell};
use crate::ast::{
    CDefinitionKind, CFileItem,
    contextual::flow_graph::{Destination, EdgeMeaning, Graph, Polarity},
};
use crate::ownership::{CSafetyError as E, numeric_flow::NumericFacts, paths::Root};
use std::collections::VecDeque;

pub(super) fn check<'ast>(facts: &NumericFacts<'ast>) -> Result<(), E> {
    let mut globals = State::default();
    for file in facts.context().files() {
        for item in file.items() {
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Object { object, .. } = definition.kind()
            {
                globals
                    .roots
                    .insert(Root::Global(object.clone()), Cell::Uninitialized);
            }
        }
    }
    let mut static_engine = Engine {
        facts,
        cursor: None,
    };
    for file in facts.context().files() {
        for item in file.items() {
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Object {
                    object,
                    initializer,
                    ..
                } = definition.kind()
            {
                let cell = static_engine.initializer(initializer, &globals)?;
                cell.complete(object.ty(), facts.context().registry())?;
                globals.roots.insert(Root::Global(object.clone()), cell);
            }
        }
    }
    // Entry into an arbitrary generated function is not program startup.
    for cell in globals.roots.values_mut() {
        *cell = Cell::Initialized;
    }
    for graph in facts.context().functions() {
        let mut entry = globals.clone();
        for file in facts.context().files() {
            for item in file.items() {
                if let CFileItem::Definition(definition) = item
                    && let CDefinitionKind::Function {
                        function,
                        parameters,
                        ..
                    } = definition.kind()
                    && function == graph.function()
                {
                    for parameter in parameters {
                        entry
                            .roots
                            .insert(Root::Parameter(parameter.clone()), Cell::Initialized);
                    }
                }
            }
        }
        function(facts, graph, entry)?;
    }
    Ok(())
}

fn function<'ast>(facts: &NumericFacts<'ast>, graph: &Graph<'ast>, entry: State) -> Result<(), E> {
    let mut incoming = vec![None; graph.nodes().len()];
    incoming[graph.entry().index()] = Some(entry);
    let mut pending = VecDeque::from([graph.entry()]);
    while let Some(point) = pending.pop_front() {
        if !facts.reachable(graph, point) {
            continue;
        }
        let mut state = incoming[point.index()].clone().ok_or(E::UnprovedStorage)?;
        let mut engine = Engine {
            facts,
            cursor: Some(facts.cursor(graph, point)?),
        };
        if engine
            .action(graph.node(point).action(), &mut state)
            .is_err()
        {
            // Solve failures carry no proof. They are checked again strictly
            // from converged inputs; a provisional failure is never success.
            state.forget_values();
        }
        for edge in graph.node(point).successors() {
            let Destination::Point(next) = edge.destination() else {
                continue;
            };
            if !facts.reachable(graph, next) {
                continue;
            }
            if let EdgeMeaning::Predicate {
                condition,
                polarity,
                ..
            } = edge.meaning()
                && engine
                    .branch(condition, polarity == Polarity::True, &state)
                    .is_ok_and(|branch| branch.is_none())
            {
                continue;
            }
            let mut outgoing = state.clone();
            if let EdgeMeaning::Predicate {
                condition,
                polarity,
                ..
            } = edge.meaning()
                && !engine
                    .refine_allocations(condition, polarity == Polarity::True, &mut outgoing)
                    .unwrap_or(true)
            {
                continue;
            }
            for scope in edge.exited_scopes() {
                outgoing.leave(scope);
            }
            let joined = match &incoming[next.index()] {
                Some(old) => old.join(&outgoing, facts.context().registry())?,
                None => outgoing,
            };
            if incoming[next.index()].as_ref() != Some(&joined) {
                incoming[next.index()] = Some(joined);
                pending.push_back(next);
            }
        }
    }
    audit::check(facts, graph, &incoming)
}
