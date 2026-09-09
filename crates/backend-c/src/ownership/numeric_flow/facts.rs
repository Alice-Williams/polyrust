//! Immutable composed facts retain the same context and actual program-point sites.
mod allocation_requests;
mod indices;
mod static_indices;
use super::{Analysis, ContextFacts, E, solve};
use crate::ast::{CRegistry, CSourceFile};
pub(in crate::ownership) use allocation_requests::{AllocationOrigin, AllocationRequest};

#[cfg(test)]
#[path = "../../tests/numeric_sites.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/static_index_extents.rs"]
mod static_index_tests;

pub(in crate::ownership) struct NumericFacts<'a> {
    context: ContextFacts<'a>,
    analysis: Analysis<'a>,
    static_indices: Vec<static_indices::Observation<'a>>,
}
impl<'a> NumericFacts<'a> {
    pub(in crate::ownership) fn check(
        registry: &'a CRegistry,
        files: &'a [CSourceFile],
    ) -> Result<Self, E> {
        let context = ContextFacts::check(registry, files)?;
        let analysis = solve::check(&context)?;
        let static_indices = static_indices::check(&context)?;
        let facts = Self {
            context,
            analysis,
            static_indices,
        };
        facts.validate_sites()?;
        Ok(facts)
    }
    fn validate_sites(&self) -> Result<(), E> {
        static_indices::validate(&self.context, &self.static_indices)?;
        if self.analysis.functions.len() != self.context.functions().len() {
            return Err(E::InvalidNumericSite);
        }
        for (facts, graph) in self.analysis.functions.iter().zip(self.context.functions()) {
            if facts.function != graph.function() || facts.incoming.len() != graph.nodes().len() {
                return Err(E::InvalidNumericSite);
            }
        }
        for obligation in &self.analysis.obligations {
            let Some(function) = self
                .analysis
                .functions
                .iter()
                .find(|function| function.function == obligation.site.function)
            else {
                return Err(E::InvalidNumericSite);
            };
            if function
                .incoming
                .get(obligation.site.point.index())
                .and_then(Option::as_ref)
                .is_none()
            {
                return Err(E::InvalidNumericSite);
            }
            let graph = self
                .context
                .functions()
                .iter()
                .find(|graph| graph.function() == obligation.site.function)
                .ok_or(E::InvalidNumericSite)?;
            super::sites::check(graph.node(obligation.site.point).action(), &obligation.kind)?;
        }
        Ok(())
    }
}
