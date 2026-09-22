//! Unsigned differences require the same independent signed-range proof as sums.
use super::wrapping_addition::shape::correct_operation;
use super::wrapping_integer::{Operation, Variant, api, chain, fixture};
use super::{dependency_fixture as f, project_c_package};
use crate::{ast::*, dialect::CStructuralRenderer};
use portable_codegen::*;

#[path = "shared_wrapping_subtraction_native.rs"]
mod native;

#[test]
fn wrapping_subtraction_proves_guards_and_rejects_signed_overflow() {
    for width in [CScalarType::I32, CScalarType::I64] {
        let build = |variant| {
            fixture::build_widths(
                701,
                &[],
                fixture::Body::Arithmetic(Operation::Subtract, variant),
                &[width],
            )
        };
        let source = build(Variant::Valid);
        assert!(correct_operation(&source, CBinaryOperator::Subtract));
        assert!(!correct_operation(&source, CBinaryOperator::Add));
        assert_eq!(api(&source).functions().count(), 1);
        for variant in [
            Variant::MissingGuard,
            Variant::ReversedGuard,
            Variant::HighLimit,
            Variant::LowLimit,
            Variant::WrongGuardValue,
            Variant::SignedArithmetic,
            Variant::UnguardedCast,
        ] {
            let source = build(variant);
            source
                .registry
                .registrations()
                .check_context(&source.files)
                .unwrap();
            assert!(!correct_operation(&source, CBinaryOperator::Subtract));
            assert!(
                project_c_package(source.registry, source.files).is_err(),
                "{width:?} {variant:?}"
            );
        }
        for variant in [
            Variant::WrongOperand,
            Variant::WrongResult,
            Variant::WrongOperation,
            Variant::ReversedOperands,
            Variant::MissingNormalization,
        ] {
            let source = build(variant);
            api(&source); // Valid C can still compute the wrong modular difference.
            assert_eq!(
                correct_operation(&source, CBinaryOperator::Subtract),
                width == CScalarType::I64 && matches!(variant, Variant::MissingNormalization),
                "{width:?} {variant:?}"
            );
        }
    }
}

#[test]
fn wrapping_subtraction_checks_recursive_children_and_depth() {
    for width in [CScalarType::I32, CScalarType::I64] {
        let build = |variant| {
            fixture::build_widths(
                701,
                &[],
                fixture::Body::Arithmetic(Operation::Subtract, variant),
                &[width],
            )
        };
        api(&build(Variant::NestedComplement(32)));
        for (variant, diagnostic) in [
            (Variant::NestedDivide, "only scalar comparisons"),
            (Variant::NestedComplement(128), "verifier budget exceeded"),
        ] {
            let source = build(variant);
            let errors = project_c_package(source.registry, source.files).unwrap_err();
            assert!(
                errors.iter().any(|e| e.message.contains(diagnostic)),
                "{errors:?}"
            );
        }
    }
}

#[test]
fn wrapping_subtraction_keeps_original_authority_and_derived_dependencies() {
    let owners = chain(Operation::Subtract, Variant::Valid);
    let foreign = fixture::build(
        703,
        &owners[0].functions().cloned().collect::<Vec<_>>(),
        fixture::Body::Forward,
    );
    let source = fixture::build(
        701,
        &[],
        fixture::Body::Arithmetic(Operation::Subtract, Variant::Valid),
    );
    assert!(
        CExpressions::new(source.registry.registrations())
            .direct(foreign.imported[0].clone())
            .is_err()
    );
    for (index, owner) in owners.iter().enumerate() {
        assert!(owner.system_libraries().is_empty());
        for file in owner.package().ast().files() {
            for standard in [super::CStdType::U32, super::CStdType::U64] {
                assert_eq!(
                    file.items()[0].unit.data.standards.contains(&standard),
                    index == 0
                );
            }
        }
        let files = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        assert_eq!(files.files().len(), 2);
        let mut bytes = 0;
        for file in files.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(
                !text.contains("runtime") && !text.contains("goto") && !text.contains("math.h")
            );
            bytes += text.len() as u64;
        }
        assert!(bytes <= crate::dialect::c_output_byte_bound(owner.package()).unwrap());
        for (function, original) in owner.functions().zip(owners[0].functions()) {
            assert!(function.stack_bound_bytes() > 0);
            if index == 1 {
                assert!(function.stack_bound_bytes() > original.stack_bound_bytes());
            }
        }
    }
    // U32 is now a public target transport type; U64 remains internal.
    let source = f::fixture(701, &[CScalarType::U64], &[], &[None]);
    assert!(project_c_package(source.registry, source.files).is_err());
}
