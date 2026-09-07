//! Certify the same capability owner that executable dispatch uses.

use std::{any::type_name, cell::RefCell, collections::BTreeSet};

use portable_build::{Bool, portable_name, typed_list, typed_program, variant};
use portable_core_ir::lower_checked;

use super::*;

thread_local! {
    static CONFIRMED: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn record<C>() {
    CONFIRMED.with_borrow_mut(|confirmed| confirmed.push(type_name::<C>()));
}

fn confirm<C: portable_build::Capability>(usage: &FeatureUse) -> JavaFeatureOwner {
    CONFIRMED.with_borrow_mut(Vec::clear);
    let decision = JavaCapabilityRegistry::default().admit(usage).unwrap();
    CONFIRMED.with_borrow(|confirmed| {
        assert_eq!(
            confirmed.as_slice(),
            [type_name::<Modules>(), type_name::<C>()]
        );
    });
    assert_eq!(decision.prerequisites, vec![CapabilityId::Modules]);
    decision.owner
}

#[test]
fn payload_free_enum_comparisons_certify_enums_not_general_equality() {
    let program = typed_program(portable_name!("enum_ownership"), |builder| {
        builder.enumeration(
            portable_name!("Choice"),
            typed_list![variant(portable_name!("FIRST"))],
            |builder, choice| {
                let builder = builder
                    .function(
                        portable_name!("same"),
                        typed_list![],
                        Bool::TYPE,
                        |body, _| {
                            let left = body.enum_variant(&choice, choice.variants().head);
                            let right = body.enum_variant(&choice, choice.variants().head);
                            body.equal(left, right)
                        },
                    )
                    .builder;
                builder
                    .function(
                        portable_name!("different"),
                        typed_list![],
                        Bool::TYPE,
                        |body, _| {
                            let left = body.enum_variant(&choice, choice.variants().head);
                            let right = body.enum_variant(&choice, choice.variants().head);
                            body.not_equal(left, right)
                        },
                    )
                    .builder
            },
        )
    });
    let core = lower_checked(program.checked_program()).expect("typed program lowers");
    let uses = collect_core_features(&core);
    let mut seen = BTreeSet::new();
    for usage in uses.iter() {
        if let CoreFeature::Operation(OperationFeature::Binary(
            op @ (CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual),
        )) = usage.feature()
        {
            assert_eq!(
                usage.shape(),
                &FeatureShape::Equality(EqualityOperandShape::PayloadFreeEnum)
            );
            assert_eq!(
                confirm::<Enums>(usage),
                JavaFeatureOwner::Mapping(CapabilityId::Enums)
            );
            seen.insert(op);
        }
    }
    assert_eq!(
        seen,
        BTreeSet::from([CoreBinaryIntrinsic::Equal, CoreBinaryIntrinsic::NotEqual])
    );
}

#[test]
fn local_reads_preserve_all_four_checked_binding_origins() {
    let mut seen = BTreeSet::new();
    for checked in [
        portable_check::v0::check_program(portable_build::interface_composition_fixture().document)
            .expect("interface fixture checks"),
        crate::tests::capability_fixtures::capability_coverage_fixture(),
    ] {
        let core = lower_checked(&checked).expect("checked fixture lowers");
        for usage in collect_core_features(&core).iter() {
            if usage.feature() != CoreFeature::Operation(OperationFeature::Local) {
                continue;
            }
            let FeatureShape::LocalBinding(kind) = usage.shape() else {
                panic!("binding origin was erased");
            };
            let decision = match kind {
                CoreLocalKind::Parameter => confirm::<Functions>(usage),
                CoreLocalKind::Let => confirm::<LocalBindings>(usage),
                CoreLocalKind::ForEach => confirm::<Loops>(usage),
                CoreLocalKind::Pattern => confirm::<PatternMatching>(usage),
            };
            assert_eq!(
                decision,
                JavaFeatureOwner::Mapping(match kind {
                    CoreLocalKind::Parameter => CapabilityId::Functions,
                    CoreLocalKind::Let => CapabilityId::LocalBindings,
                    CoreLocalKind::ForEach => CapabilityId::Loops,
                    CoreLocalKind::Pattern => CapabilityId::PatternMatching,
                })
            );
            seen.insert(*kind);
        }
    }
    assert_eq!(
        seen,
        BTreeSet::from([
            CoreLocalKind::Parameter,
            CoreLocalKind::Let,
            CoreLocalKind::ForEach,
            CoreLocalKind::Pattern
        ])
    );
}

#[test]
fn constant_enum_comparison_preserves_operand_type_through_references() {
    use portable_build::{ModuleBuilder, Operation, Type, Visibility};

    let mut module = ModuleBuilder::new("constant_enum_ownership");
    let (choice, (first, ())) =
        module.enumeration("Choice", Visibility::Public, vec![], |enumeration| {
            enumeration.variant("FIRST", vec![], |_| {})
        });
    let selected = module.constant(
        "SELECTED",
        Visibility::Public,
        vec![],
        Type::named(choice),
        |body| body.constant_enum(choice, first, []),
    );
    for (name, operation) in [
        ("SAME", Operation::Equal),
        ("DIFFERENT", Operation::NotEqual),
    ] {
        module.constant(name, Visibility::Public, vec![], Type::bool(), |body| {
            let left = body.constant_reference(selected);
            let right = body.constant_enum(choice, first, []);
            body.constant_intrinsic(operation, [left, right])
        });
    }
    let checked = module.finish().expect("constant enum fixture checks");
    let core = lower_checked(&checked).expect("constant enum fixture lowers");
    let mut count = 0;
    for usage in collect_core_features(&core).iter() {
        if matches!(
            usage.feature(),
            CoreFeature::Operation(OperationFeature::Binary(
                CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual
            ))
        ) {
            assert_eq!(
                usage.shape(),
                &FeatureShape::Equality(EqualityOperandShape::PayloadFreeEnum)
            );
            assert_eq!(
                confirm::<Enums>(usage),
                JavaFeatureOwner::Mapping(CapabilityId::Enums)
            );
            count += 1;
        }
    }
    assert_eq!(count, 2);
}
