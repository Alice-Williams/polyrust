//! Borrowed checked context; graphs and inputs cannot be mutated through facts.
use super::{CSafetyError as E, sequencing};
use crate::ast::{
    CDefinitionKind, CFileItem, CRegistry, CSourceFile, contextual::flow_graph::Graph,
};

pub(super) struct ContextFacts<'a> {
    registry: &'a CRegistry,
    files: &'a [CSourceFile],
    functions: Vec<Graph<'a>>,
}

impl<'a> ContextFacts<'a> {
    pub(super) fn check(registry: &'a CRegistry, files: &'a [CSourceFile]) -> Result<Self, E> {
        registry.check_constants_and_layout(files)?;
        let mut functions = Vec::new();
        for file in files {
            for item in file.items() {
                if let CFileItem::Definition(value) = item
                    && let CDefinitionKind::Function { body, .. } = value.kind()
                {
                    functions.push(Graph::build(body)?);
                }
            }
        }
        let facts = Self {
            registry,
            files,
            functions,
        };
        sequencing::check(&facts)?;
        Ok(facts)
    }
    pub(super) const fn registry(&self) -> &'a CRegistry {
        self.registry
    }
    pub(super) const fn files(&self) -> &'a [CSourceFile] {
        self.files
    }
    pub(super) fn functions(&self) -> &[Graph<'a>] {
        &self.functions
    }
}

impl CRegistry {
    /// Composes contextual, constant/layout and call-sequencing diagnostics.
    /// Range, storage, call effects, linking and rendering remain unproved.
    /// No mutable graph or source-validity certificate escapes this method.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::context_facts::ContextFacts;
    /// fn forge<'a>() -> ContextFacts<'a> { ContextFacts { registry: todo!(), files: &[], functions: vec![] } }
    /// ```
    ///
    /// ```compile_fail
    /// use portable_backend_c::ast::contextual::flow_graph::{Graph, Point};
    /// fn corrupt(graph: &mut Graph<'_>) { graph.entry = Point(0); }
    /// ```
    pub fn check_sequencing_and_control(&self, files: &[CSourceFile]) -> Result<(), E> {
        let facts = ContextFacts::check(self, files)?;
        // Retain the borrowed provenance of this diagnostic boundary. Later
        // analyses consume these accessors, never a caller's second projection.
        debug_assert!(std::ptr::eq(facts.registry(), self));
        debug_assert!(std::ptr::eq(facts.files(), files));
        Ok(())
    }
}
