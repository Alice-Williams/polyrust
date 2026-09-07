//! Admission regressions independent of invocation strategy certificates.
use super::*;
use portable_build::{ModuleBuilder, Visibility};
use portable_core_ir::lower_checked;

#[test]
fn legacy_zero_variant_enum_is_rejected_before_java_lowering() {
    let mut module = ModuleBuilder::new("empty_enum");
    module.enumeration("Empty", Visibility::Public, vec![], |_| {});
    let checked = module.finish().expect("legacy checker admits empty enum");
    let core = lower_checked(&checked).expect("checked empty enum");
    let errors = JavaCapabilityRegistry::default().select(&core).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("zero-variant enums"));
    assert_eq!(errors[0].target.as_deref(), Some("org.polyrust.java"));
}

#[test]
fn admission_separates_native_owner_from_fallible_prerequisites() {
    use portable_core_ir::{CoreBinaryIntrinsic as B, CoreUnaryIntrinsic as U};
    let checked = crate::tests::capability_fixtures::capability_coverage_fixture();
    let core = lower_checked(&checked).unwrap();
    let selection = JavaCapabilityRegistry::default().select(&core).unwrap();
    for (operation, owner, fallible) in [
        (
            OperationFeature::Unary(U::BoolNot),
            CapabilityId::BooleanLogic,
            false,
        ),
        (
            OperationFeature::Unary(U::IntNegWrapping),
            CapabilityId::WrappingIntegerArithmetic,
            false,
        ),
        (
            OperationFeature::Unary(U::IntNegChecked),
            CapabilityId::CheckedIntegerArithmetic,
            true,
        ),
        (
            OperationFeature::Binary(B::Equal),
            CapabilityId::Equality,
            false,
        ),
        (
            OperationFeature::Binary(B::IntAddWrapping),
            CapabilityId::WrappingIntegerArithmetic,
            false,
        ),
        (
            OperationFeature::Binary(B::IntAddChecked),
            CapabilityId::CheckedIntegerArithmetic,
            true,
        ),
        (
            OperationFeature::Binary(B::StringConcat),
            CapabilityId::StringConcatenation,
            false,
        ),
    ] {
        let actual = selection
            .selected
            .iter()
            .find(|entry| entry.usage.feature() == CoreFeature::Operation(operation))
            .expect("operation exercised by fixture");
        assert_eq!(actual.owner, JavaFeatureOwner::Mapping(owner));
        let expected = if fallible {
            vec![CapabilityId::Modules, CapabilityId::ResultPropagation]
        } else {
            vec![CapabilityId::Modules]
        };
        assert_eq!(actual.prerequisites, expected);
    }
}

#[test]
fn structural_assembly_never_impersonates_a_modules_mapping() {
    let checked = crate::tests::capability_fixtures::capability_coverage_fixture();
    let core = lower_checked(&checked).unwrap();
    let selection = JavaCapabilityRegistry::default().select(&core).unwrap();
    let interfaces =
        portable_check::v0::check_program(portable_build::interface_composition_fixture().document)
            .unwrap();
    let interface_core = lower_checked(&interfaces).unwrap();
    let interface_selection = JavaCapabilityRegistry::default()
        .select(&interface_core)
        .unwrap();
    for (feature, reason) in [
        (
            CoreFeature::Control(ControlFeature::Block),
            JavaStructuralAdmission::BlockAssembly,
        ),
        (
            CoreFeature::Control(ControlFeature::Evaluate),
            JavaStructuralAdmission::EvaluationSequence,
        ),
        (
            CoreFeature::Operation(OperationFeature::Literal),
            JavaStructuralAdmission::LiteralDispatch,
        ),
        (
            CoreFeature::Control(ControlFeature::Match),
            JavaStructuralAdmission::MatchDispatch,
        ),
        (
            CoreFeature::Ownership(OwnershipFeature::OnceLeftToRight),
            JavaStructuralAdmission::OwnershipContract,
        ),
    ] {
        let entry = selection
            .selected
            .iter()
            .chain(&interface_selection.selected)
            .find(|entry| entry.usage.feature() == feature)
            .unwrap_or_else(|| panic!("structural feature {feature:?} in fixtures"));
        assert_eq!(entry.owner, JavaFeatureOwner::Structural(reason));
        if reason == JavaStructuralAdmission::MatchDispatch {
            assert_eq!(
                entry.prerequisites,
                vec![CapabilityId::Modules, CapabilityId::PatternMatching]
            );
        } else {
            assert_eq!(entry.prerequisites, vec![CapabilityId::Modules]);
        }
    }
    let mut incomplete = selection.clone();
    incomplete.selected.pop();
    assert!(incomplete.validate_for(&core, java_capabilities()).is_err());
    let mut extra = selection.clone();
    extra.selected.push(selection.selected[0].clone());
    assert!(extra.validate_for(&core, java_capabilities()).is_err());
}
