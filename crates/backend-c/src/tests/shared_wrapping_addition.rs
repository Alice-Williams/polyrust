//! Target-only signed wrapping addition: checked normal ASTs, no helper/runtime.
use super::{
    CDependencyApi, CDependencyFunction, CDialect, dependency_fixture as f, project_c_package,
};
use crate::{ast::*, dialect::CStructuralRenderer};
use portable_codegen::*;

#[path = "shared_wrapping_addition_body.rs"]
mod body;
use body::Variant;
#[path = "shared_wrapping_addition_contracts.rs"]
mod contracts;
#[path = "shared_wrapping_addition_fixture.rs"]
mod fixture;
#[path = "shared_wrapping_addition_native.rs"]
mod native;
#[path = "shared_wrapping_addition_shape.rs"]
mod shape;

fn api(source: &f::Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, f::linked(source)).unwrap(),
    )
    .unwrap()
}

#[test]
fn wrapping_addition_proves_both_signed_casts_and_rejects_unsafe_guards() {
    for width in [CScalarType::I32, CScalarType::I64] {
        for variant in [
            Variant::Valid,
            Variant::MissingGuard,
            Variant::ReversedGuard,
            Variant::HighLimit,
            Variant::LowLimit,
            Variant::WrongGuardValue,
            Variant::SignedAdd,
            Variant::UnguardedCast,
        ] {
            let source =
                fixture::build_widths(701, &[], fixture::Body::Addition(variant), &[width]);
            source
                .registry
                .registrations()
                .check_context(&source.files)
                .unwrap();
            let result = project_c_package(source.registry.clone(), source.files.clone());
            if matches!(variant, Variant::Valid) {
                assert!(result.is_ok(), "{variant:?}: {result:?}");
                assert_eq!(api(&source).functions().count(), 1);
            } else {
                let errors = result.expect_err("unsafe integer construction must reject");
                assert!(
                    errors.iter().any(|error| {
                        if matches!(variant, Variant::SignedAdd) {
                            error.message.contains("only scalar comparisons")
                        } else {
                            error.message.contains(
                                "C numeric conversion is outside its proven integer range",
                            )
                        }
                    }),
                    "{variant:?}: {errors:?}"
                );
            }
        }
    }
}

#[test]
fn wrapping_addition_internal_unsigned_values_do_not_admit_unsigned_source_signatures() {
    for scalar in [CScalarType::U32, CScalarType::U64] {
        let source = f::fixture(701, &[scalar], &[], &[None]);
        let errors = project_c_package(source.registry, source.files).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("scalar parameters"))
        );
    }
}

fn chain(variant: Variant) -> Vec<CDependencyApi> {
    let producer = api(&fixture::build(701, &[], fixture::Body::Addition(variant)));
    let imports: Vec<CDependencyFunction> = producer.functions().cloned().collect();
    let consumer = api(&fixture::build(702, &imports, fixture::Body::Forward));
    vec![producer, consumer]
}

#[test]
fn wrapping_addition_keeps_original_authority_and_bounded_runtime_free_packages() {
    let owners = chain(Variant::Valid);
    let foreign = fixture::build(
        703,
        &owners[0].functions().cloned().collect::<Vec<_>>(),
        fixture::Body::Forward,
    );
    let source = fixture::build(701, &[], fixture::Body::Addition(Variant::Valid));
    assert!(
        CExpressions::new(source.registry.registrations())
            .direct(foreign.imported[0].clone())
            .is_err()
    );
    for (index, owner) in owners.iter().enumerate() {
        assert!(owner.system_libraries().is_empty());
        let files = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        assert_eq!(files.files().len(), 2);
        let mut bytes = 0;
        for file in files.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(!text.contains("runtime") && !text.contains("goto"));
            assert!(!text.contains("math.h") && !text.contains("memcpy"));
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
}
