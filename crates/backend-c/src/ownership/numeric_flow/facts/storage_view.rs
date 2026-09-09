//! Borrowed context and actual-occurrence queries for the storage consumer.
use super::NumericFacts;
use crate::ast::{
    CPlace, CValue,
    contextual::flow_graph::{Graph, Point},
};
use crate::ownership::layout::Layouts;
use crate::ownership::numeric_flow::{Engine, Mode, State};
use crate::ownership::{CSafetyError as E, context_facts::ContextFacts};
use std::collections::BTreeSet;

#[derive(Clone)]
pub(in crate::ownership) struct Cursor<'ast> {
    registry: &'ast crate::ast::CRegistry,
    state: State<'ast>,
}
impl<'ast> Cursor<'ast> {
    pub(in crate::ownership) fn branch(
        &self,
        value: &'ast CValue,
        truth: bool,
    ) -> Result<Option<Self>, E> {
        let mut engine = Engine {
            registry: self.registry,
            layouts: Layouts::new(self.registry),
            addresses: BTreeSet::new(),
            mode: Mode::Derive,
            obligations: vec![],
        };
        Ok(engine.refine(&self.state, value, truth)?.map(|state| Self {
            registry: self.registry,
            state,
        }))
    }
}

impl<'ast> NumericFacts<'ast> {
    pub(in crate::ownership) fn context(&self) -> &ContextFacts<'ast> {
        &self.context
    }

    pub(in crate::ownership) fn reachable(&self, graph: &Graph<'ast>, point: Point) -> bool {
        self.analysis
            .functions
            .iter()
            .find(|facts| facts.function == graph.function())
            .and_then(|facts| facts.incoming.get(point.index()))
            .is_some_and(Option::is_some)
    }

    pub(in crate::ownership) fn cursor(
        &self,
        graph: &Graph<'ast>,
        point: Point,
    ) -> Result<Cursor<'ast>, E> {
        let state = self
            .analysis
            .functions
            .iter()
            .find(|facts| facts.function == graph.function())
            .and_then(|facts| facts.incoming.get(point.index()))
            .and_then(Option::as_ref)
            .ok_or(E::InvalidNumericSite)?
            .clone();
        Ok(Cursor {
            registry: self.context.registry(),
            state,
        })
    }

    /// Union every observation of this exact immutable AST occurrence. Never
    /// choose a more favorable site or substitute an equal cloned expression.
    pub(in crate::ownership) fn index_bounds(&self, place: &CPlace) -> Result<(u64, u64), E> {
        let mut combined: Option<(u64, u64)> = None;
        for observation in self.indices().filter(|o| std::ptr::eq(o.place(), place)) {
            let (first, last) = observation.nonwrapping_bounds()?;
            let first = u64::try_from(first).map_err(|_| E::IndexOutOfBounds)?;
            let last = u64::try_from(last).map_err(|_| E::IndexOutOfBounds)?;
            combined = Some(match combined {
                Some((a, b)) => (a.min(first), b.max(last)),
                None => (first, last),
            });
        }
        combined.ok_or(E::InvalidNumericSite)
    }
}
