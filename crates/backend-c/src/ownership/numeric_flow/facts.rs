//! Immutable composed facts retain the same context and actual program-point sites.
use super::{Analysis, ContextFacts, E, solve};
use crate::ast::{CRegistry, CSourceFile};

#[cfg(test)]
#[path = "../../tests/numeric_sites.rs"]
mod tests;

pub(in crate::ownership) struct NumericFacts<'a> {
    context: ContextFacts<'a>,
    analysis: Analysis<'a>,
}
impl<'a> NumericFacts<'a> {
    pub(in crate::ownership) fn check(
        registry: &'a CRegistry,
        files: &'a [CSourceFile],
    ) -> Result<Self, E> {
        let context = ContextFacts::check(registry, files)?;
        let analysis = solve::check(&context)?;
        let facts = Self { context, analysis };
        facts.validate_sites()?;
        Ok(facts)
    }
    fn validate_sites(&self) -> Result<(), E> {
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
