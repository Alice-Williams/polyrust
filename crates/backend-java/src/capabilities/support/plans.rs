//! Closed, output-typed plans selected before a Java mapping runs.
pub(crate) mod expressions;
pub(crate) mod intrinsics;
pub(crate) mod values;
pub(crate) mod sealed {
    pub trait JavaMappingPlan {}
}

use super::JavaMappingOutput;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaRepresentation {
    Direct,
    RuntimeHelper,
    StructuredControl,
    Declaration,
    TaggedValue,
    InterfaceDispatch,
    Erased,
}

/// Each associated plan can verify only its own typed output category.
pub trait JavaMappingPlan: sealed::JavaMappingPlan {
    type Output: JavaMappingOutput;
    fn representation(&self) -> JavaRepresentation;
    fn verify_output(&self, output: &Self::Output) -> bool;
}
