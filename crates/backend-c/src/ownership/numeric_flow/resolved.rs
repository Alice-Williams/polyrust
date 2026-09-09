//! Private memory resolution hook; never a caller-supplied proof or renderer input.
use super::{E, Engine, Mode, State, storage::Key};
use crate::ast::{
    CPlace,
    contextual::flow_graph::{Graph, Point},
};
use crate::ownership::{context_facts::ContextFacts, layout::Layouts, loops};

pub(in crate::ownership) trait Places<'ast> {
    fn resolve(&self, place: &'ast CPlace, state: &State<'ast>) -> Result<Key, E>;
    fn truth(
        &self,
        value: &'ast crate::ast::CValue,
        state: &State<'ast>,
    ) -> Result<Option<bool>, E>;
    fn roots(&self) -> std::collections::BTreeSet<super::Root>;
}

impl<'ast, 'resolver> Engine<'ast, 'resolver> {
    pub(in crate::ownership) fn request(
        &mut self,
        context: &ContextFacts<'ast>,
        graph: &Graph<'ast>,
        point: Point,
        call: &'ast crate::ast::CCall,
        state: &mut State<'ast>,
    ) -> Result<super::AllocationRequest, E> {
        let bytes = self.numeric(
            call.arguments().first().ok_or(E::ExpectedNumericValue)?,
            state,
        )?;
        super::AllocationRequest::actual(context, graph, point, call, &bytes)
    }
    pub(in crate::ownership) fn composed(
        context: &ContextFacts<'ast>,
        resolver: &'resolver dyn Places<'ast>,
        solving: bool,
    ) -> Result<Self, E> {
        Ok(Self {
            registry: context.registry(),
            layouts: Layouts::new(context.registry()),
            addresses: super::storage::addresses(context)?,
            mode: if solving { Mode::Solve } else { Mode::Derive },
            obligations: vec![],
            resolver: Some(resolver),
        })
    }

    pub(in crate::ownership) fn verify_action(
        &mut self,
        graph: &Graph<'ast>,
        point: Point,
        state: &mut State<'ast>,
    ) -> Result<(), E> {
        self.mode = Mode::Verify(super::Site {
            function: graph.function(),
            point,
        });
        self.action(graph.node(point).action(), state)?;
        for obligation in &self.obligations {
            super::sites::check(graph.node(point).action(), &obligation.kind)?;
        }
        self.check_obligations()
    }

    pub(super) fn resolve(
        &mut self,
        place: &'ast CPlace,
        state: &State<'ast>,
    ) -> Result<Option<Key>, E> {
        match self.resolver {
            Some(resolver) => resolver.resolve(place, state).map(Some),
            None => Ok(Key::place(place, &mut self.layouts)),
        }
    }

    pub(super) fn exact_place(
        &mut self,
        place: &'ast CPlace,
        state: &State<'ast>,
    ) -> Result<Option<Key>, E> {
        Ok(self.resolve(place, state)?.filter(Key::exact))
    }
}

pub(in crate::ownership) fn progress<'ast>(
    state: &mut State<'ast>,
    graph: &Graph<'ast>,
    point: Point,
    loops: &[loops::LoopEvidence<'ast>],
) -> Result<bool, E> {
    super::solve::progress::refine(state, graph, point, loops)
}
