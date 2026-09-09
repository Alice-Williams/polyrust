//! Finite per-iteration path counting, including nested cycles and early exits.
use super::{E, LoopEvidence, StepPhase, shape};
use crate::ast::{
    CRegistry,
    contextual::flow_graph::{BranchOwner, Destination, EdgeMeaning, Graph, Polarity},
};
use crate::ownership::{constants, layout::Layouts};
use std::collections::{BTreeSet, VecDeque};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Steps {
    Zero,
    One,
}

pub(super) fn check(
    registry: &CRegistry,
    graph: &Graph<'_>,
    evidence: &LoopEvidence<'_>,
) -> Result<Vec<Option<StepPhase>>, E> {
    let entry = graph
        .nodes()
        .iter()
        .flat_map(|node| node.successors())
        .find_map(|edge| match edge.meaning() {
            EdgeMeaning::Predicate {
                owner: BranchOwner::Loop(identity),
                polarity: Polarity::True,
                ..
            } if identity == evidence.identity => Some(edge.destination()),
            _ => None,
        })
        .ok_or(E::InvalidCountedLoop)?;
    let Destination::Point(entry) = entry else {
        return Err(E::InvalidCountedLoop);
    };
    let mut layouts = Layouts::new(registry);
    let mut pending = VecDeque::from([(entry, Steps::Zero)]);
    let mut visited = BTreeSet::new();
    while let Some((point, mut steps)) = pending.pop_front() {
        if !visited.insert((point, steps)) {
            continue;
        }
        let node = graph.node(point);
        if node.origin().is_some_and(|statement| {
            evidence
                .steps
                .iter()
                .any(|step| std::ptr::eq(*step, statement))
        }) {
            steps = match steps {
                Steps::Zero => Steps::One,
                Steps::One => return Err(E::RepeatedLoopStep),
            };
        }
        for edge in node.successors() {
            if let EdgeMeaning::Predicate {
                condition,
                polarity,
                ..
            } = edge.meaning()
                && let Ok(value) = constants::evaluate(&mut layouts, condition)
                && value.truth() != (polarity == Polarity::True)
            {
                continue;
            }
            if matches!(edge.meaning(),
                EdgeMeaning::Backedge(id) | EdgeMeaning::Continue(id) if id == evidence.identity)
            {
                if steps != Steps::One {
                    return Err(E::MissingLoopStep);
                }
                continue;
            }
            let Destination::Point(next) = edge.destination() else {
                continue;
            };
            // Break/Return/cleanup exits need no step. A nested loop's own
            // backedge stays inside this region, preserving the outer count.
            if shape::contains(evidence.body.scope(), graph.node(next).scope()) {
                pending.push_back((next, steps));
            }
        }
    }
    let mut phases = vec![None; graph.nodes().len()];
    for (point, steps) in visited {
        let phase = match steps {
            Steps::Zero => StepPhase::Before,
            Steps::One => StepPhase::After,
        };
        let old = &mut phases[point.index()];
        *old = Some(match *old {
            None => phase,
            Some(old) if old == phase => phase,
            Some(_) => StepPhase::Either,
        });
    }
    Ok(phases)
}
