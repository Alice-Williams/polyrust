//! One executable source-capability contract shared by compiler adapters.

/// Session-bound compiler input, independent of a target AST or reader.
pub(crate) trait Capability {
    type Input<'tcx>;
}

/// A stored executable implementation, never an independent support flag.
pub(crate) trait Mapping: Copy {
    type Capability: Capability;
    type Context<'tcx>;
    type Output;

    fn lower<'tcx>(
        &self,
        context: &mut Self::Context<'tcx>,
        input: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String>;
}

pub(crate) trait Supports<C: Capability> {
    type Mapping: Mapping<Capability = C>;
    fn mapping(&self) -> Self::Mapping;
}
