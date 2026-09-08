//! Exact checked match prerequisites, independently mutated before lowering.
use super::{
    JavaCapabilityRegistry,
    admission::{JavaFeatureOwner, JavaStructuralAdmission},
};
use crate::capabilities::java_capabilities;
use portable_build::CapabilityId;
use portable_codegen::{ControlFeature, CoreFeature, OperationFeature};
use portable_core_ir::lower_checked;

#[test]
fn enum_match_admission_requires_the_mapping_actually_used_by_lowering() {
    for (payload, wildcard) in [(false, false), (true, false), (false, true), (true, true)] {
        let checked = crate::tests::match_dispatch::fixture(payload, wildcard);
        let core = lower_checked(&checked).unwrap();
        let selection = JavaCapabilityRegistry::default().select(&core).unwrap();
        for feature in [
            CoreFeature::Control(ControlFeature::Match),
            CoreFeature::Operation(OperationFeature::Match),
            CoreFeature::Control(ControlFeature::EnumPattern),
        ] {
            let index = selection
                .selected
                .iter()
                .position(|entry| entry.usage.feature() == feature)
                .unwrap();
            let entry = &selection.selected[index];
            let owner = if payload
                || (wildcard && feature != CoreFeature::Control(ControlFeature::EnumPattern))
            {
                CapabilityId::PatternMatching
            } else {
                CapabilityId::Enums
            };
            assert_eq!(
                entry.owner,
                JavaFeatureOwner::Structural(JavaStructuralAdmission::MatchDispatch)
            );
            assert_eq!(entry.prerequisites, vec![CapabilityId::Modules, owner]);

            let mut missing = selection.clone();
            missing.selected[index].prerequisites.pop();
            assert!(missing.validate_for(&core, java_capabilities()).is_err());
            let mut wrong = selection.clone();
            wrong.selected[index].prerequisites[1] = if owner == CapabilityId::Enums {
                CapabilityId::PatternMatching
            } else {
                CapabilityId::Enums
            };
            assert!(wrong.validate_for(&core, java_capabilities()).is_err());
        }
    }
}
