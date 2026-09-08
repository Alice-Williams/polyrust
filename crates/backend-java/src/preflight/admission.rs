//! Registration admission has no target representation strategy.
use portable_build::CapabilityId;
use portable_codegen::FeatureUse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JavaFeatureAdmission {
    pub usage: FeatureUse,
    pub owner: JavaFeatureOwner,
    pub prerequisites: Vec<CapabilityId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JavaFeatureOwner {
    Mapping(CapabilityId),
    Structural(JavaStructuralAdmission),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JavaStructuralAdmission {
    BlockAssembly,
    EvaluationSequence,
    OwnershipContract,
    LiteralDispatch,
    MatchDispatch,
}
