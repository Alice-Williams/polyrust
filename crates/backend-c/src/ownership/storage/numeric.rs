//! Resolve numeric places using the same current memory snapshot, recursively.
use super::{Engine, state::State};
use crate::ast::{CCall, CPlace, CValue};
use crate::ownership::{
    CSafetyError as E,
    numeric_flow::{self, AllocationRequest, Number, Places},
    paths::Key,
};

pub(super) struct Resolver<'view, 'facts, 'ast> {
    pub(super) engine: &'view Engine<'facts, 'ast>,
    pub(super) memory: &'view State,
}
impl<'ast> Places<'ast> for Resolver<'_, '_, 'ast> {
    fn truth(
        &self,
        value: &'ast CValue,
        numeric: &numeric_flow::State<'ast>,
    ) -> Result<Option<bool>, E> {
        let mut engine = self.engine.clone();
        engine.numeric = Some(numeric.clone());
        engine.pointer_truth(value, self.memory)
    }
    fn roots(&self) -> std::collections::BTreeSet<crate::ownership::paths::Root> {
        self.memory.roots.keys().cloned().collect()
    }
    fn resolve(&self, place: &'ast CPlace, numeric: &numeric_flow::State<'ast>) -> Result<Key, E> {
        let mut engine = self.engine.clone();
        engine.numeric = Some(numeric.clone());
        engine.place(place, self.memory)
    }
}
impl<'ast> Engine<'_, 'ast> {
    pub(super) fn index_binding(
        &self,
        value: &'ast CValue,
        memory: &State,
    ) -> Result<Option<Key>, E> {
        let resolver = Resolver {
            engine: self,
            memory,
        };
        let mut checker = numeric_flow::Engine::composed(self.context, &resolver, false)?;
        checker.index_binding(value, self.numeric.as_ref().ok_or(E::InvalidNumericSite)?)
    }
    pub(super) fn index_below_count(
        &self,
        index: &'ast CValue,
        count: &crate::ast::CBufferCountRef,
        memory: &State,
    ) -> Result<bool, E> {
        let resolver = Resolver {
            engine: self,
            memory,
        };
        let mut checker = numeric_flow::Engine::composed(self.context, &resolver, false)?;
        checker.below_count(
            index,
            count,
            self.numeric.as_ref().ok_or(E::InvalidNumericSite)?,
        )
    }
    pub(super) fn number(&self, value: &'ast CValue, memory: &State) -> Result<Number<'ast>, E> {
        let resolver = Resolver {
            engine: self,
            memory,
        };
        let mut checker = numeric_flow::Engine::composed(self.context, &resolver, false)?;
        checker.numeric(value, &mut self.numeric.clone().unwrap_or_default())
    }
    pub(super) fn allocation_request(
        &self,
        call: &'ast CCall,
        memory: &State,
    ) -> Result<AllocationRequest, E> {
        let (graph, point) = self.site.ok_or(E::InvalidNumericSite)?;
        let resolver = Resolver {
            engine: self,
            memory,
        };
        let mut checker = numeric_flow::Engine::composed(self.context, &resolver, false)?;
        checker.request(
            self.context,
            graph,
            point,
            call,
            &mut self.numeric.clone().ok_or(E::InvalidNumericSite)?,
        )
    }
}
