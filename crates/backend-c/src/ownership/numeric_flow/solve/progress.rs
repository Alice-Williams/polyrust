//! Phase facts come from 03B's actual graph, never from counter spelling.
use crate::ast::CScalarType;
use crate::ast::contextual::flow_graph::{Graph, Point};
use crate::ownership::{
    CSafetyError as E,
    loops::{LoopEvidence, StepPhase},
    numeric_flow::{
        state::{Number, State},
        storage::Key,
    },
    ranges::NumericDomain,
};

pub(in crate::ownership::numeric_flow) fn refine<'a>(
    state: &mut State<'a>,
    graph: &Graph<'a>,
    point: Point,
    loops: &[LoopEvidence<'a>],
) -> Result<bool, E> {
    for evidence in loops {
        let Some(phase) = evidence.phase(graph.function(), point) else {
            continue;
        };
        let counter_key = Key::local(evidence.counter());
        let bound_key = Key::local(evidence.bound());
        let mut counter = state
            .read(&counter_key)
            .cloned()
            .unwrap_or(Number::domain(NumericDomain::full(CScalarType::Size)?));
        let mut bound = state
            .read(&bound_key)
            .cloned()
            .unwrap_or(Number::domain(NumericDomain::full(CScalarType::Size)?));
        let Some((counter_min, _)) = counter.domain.integer_bounds() else {
            return Ok(false);
        };
        let Some((_, bound_max)) = bound.domain.integer_bounds() else {
            return Ok(false);
        };
        let strict = i128::from(phase == StepPhase::Before);
        counter.domain = counter.domain.restrict_integer(0, bound_max - strict)?;
        bound.domain = bound
            .domain
            .restrict_integer((counter_min + strict).max(1), i128::MAX)?;
        if counter.domain.is_empty() || bound.domain.is_empty() {
            return Ok(false);
        }
        state.set(counter_key, counter);
        state.set(bound_key, bound);
    }
    Ok(true)
}
