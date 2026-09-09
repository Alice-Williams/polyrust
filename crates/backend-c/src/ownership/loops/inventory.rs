//! Exhaustive actual writes/addresses; supplied progress records cannot hide any.
use super::{E, LoopEvidence, shape};
use crate::ast::{
    CLocalRef, CPlace, CPlaceKind,
    contextual::{
        access_statements,
        access_walk::{Access, Visitor},
        flow_graph::{Action, Graph},
    },
};
use crate::ownership::context_facts::ContextFacts;
use std::collections::BTreeSet;

pub(super) fn check<'a>(
    context: &ContextFacts<'a>,
    graph: &Graph<'a>,
    loops: &mut [LoopEvidence<'a>],
) -> Result<(), E> {
    let mut counters = BTreeSet::new();
    for value in loops.iter() {
        if !counters.insert(value.progress.counter()) {
            return Err(E::SharedLoopCounter);
        }
    }
    let bodies: Vec<_> = loops.iter().map(|value| value.body.scope()).collect();
    for value in loops {
        let mut aliases = Aliases {
            counter: value.progress.counter(),
            bound: value.progress.bound(),
        };
        for file in context.files() {
            access_statements::file(&mut aliases, file)?;
        }
        for node in graph.nodes() {
            let Action::Assign(place, update) = node.action() else {
                continue;
            };
            if !matches!(place.kind(), CPlaceKind::Local(v) if v == value.progress.counter()) {
                continue;
            }
            if !shape::contains(value.body.scope(), node.scope())
                || bodies.iter().any(|body| {
                    *body != value.body.scope()
                        && shape::contains(value.body.scope(), body)
                        && shape::contains(body, node.scope())
                })
                || !shape::step(update, value.progress.counter())
            {
                return Err(E::InvalidLoopMutation);
            }
            value
                .steps
                .push(node.origin().ok_or(E::InvalidLoopMutation)?);
        }
    }
    Ok(())
}

struct Aliases<'a> {
    counter: &'a CLocalRef,
    bound: &'a CLocalRef,
}
impl Visitor for Aliases<'_> {
    type Error = E;
    fn place(&mut self, place: &CPlace, access: Access) -> Result<(), E> {
        if let CPlaceKind::Local(value) = place.kind() {
            if access == Access::Address && (value == self.counter || value == self.bound) {
                return Err(E::LoopAddressEscape);
            }
            if access == Access::Write && value == self.bound {
                return Err(E::InvalidLoopMutation);
            }
        }
        Ok(())
    }
}
