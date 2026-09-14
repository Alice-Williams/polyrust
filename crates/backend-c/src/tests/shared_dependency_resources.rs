//! Compose real immutable certificate witnesses, not caller-written costs.
use super::{
    CDependencyApi, CDependencyFunction, CDialect, dependency_fixture as fixture, resources,
};
use crate::ast::CScalarType;
use portable_codegen::{certify_resolved_package, render_certified_package};

fn api(input: &fixture::Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, fixture::linked(input)).unwrap(),
    )
    .unwrap()
}

fn functions(api: &CDependencyApi) -> Vec<CDependencyFunction> {
    api.functions().cloned().collect()
}

fn chain(crate_id: u64, dependency: &CDependencyApi) -> fixture::Fixture {
    fixture::fixture(
        crate_id,
        &[CScalarType::I32],
        &functions(dependency),
        &[Some(0)],
    )
}

#[test]
fn direct_and_transitive_costs_include_the_dependency_but_frames_stay_owned() {
    for scalar in [CScalarType::I32, CScalarType::Bool] {
        let leaf = fixture::api(10, &[scalar]);
        let middle = fixture::fixture(20, &[scalar], &functions(&leaf), &[Some(0)]);
        let middle_linked = fixture::linked(&middle);
        let middle_measure = resources::measure_package(&middle_linked).unwrap();
        assert_eq!(middle_measure.total.function_frames.len(), 1);
        assert_eq!(
            middle_measure.total.frame_bound,
            middle_measure.total.function_frames[&middle.functions[0]]
                + leaf.functions().next().unwrap().stack_bound_bytes()
        );
        let middle = api(&middle);
        assert_eq!(
            middle.functions().next().unwrap().stack_bound_bytes(),
            middle_measure.total.frame_bound
        );
        let consumer = fixture::fixture(30, &[scalar], &functions(&middle), &[Some(0)]);
        let linked = fixture::linked(&consumer);
        let measured = resources::measure_package(&linked).unwrap();
        assert_eq!(measured.total.function_frames.len(), 1);
        assert_eq!(
            measured.total.frame_bound,
            measured.total.function_frames[&consumer.functions[0]]
                + middle_measure.total.frame_bound
        );
        let certified = certify_resolved_package(&CDialect, linked).unwrap();
        assert_eq!(
            render_certified_package(&super::CStructuralRenderer, &certified)
                .unwrap()
                .files()
                .len(),
            2
        );
        assert!(CDependencyApi::from_certificate(certified).is_ok());
    }
}

#[test]
fn unused_imports_keep_the_old_no_call_cost_and_do_not_copy_foreign_frames() {
    let leaf = fixture::api(10, &[CScalarType::I32]);
    let plain = fixture::fixture(20, &[CScalarType::I32], &[], &[None]);
    let unused = fixture::fixture(20, &[CScalarType::I32], &functions(&leaf), &[None]);
    let before = resources::measure_package(&fixture::linked(&plain)).unwrap();
    let linked = fixture::linked(&unused);
    let after = resources::measure_package(&linked).unwrap();
    assert_eq!(before.total.frame_bound, after.total.frame_bound);
    assert_eq!(before.total.source_bound, after.total.source_bound);
    assert_eq!(after.total.function_frames.len(), 1);
    assert!(certify_resolved_package(&CDialect, linked).is_ok());
}

#[test]
fn shared_certificate_diamonds_are_valid_but_independent_same_crate_certificates_reject() {
    let first_leaf = fixture::api(10, &[CScalarType::I32]);
    let other_leaf = fixture::api(10, &[CScalarType::I32]);
    let left = api(&chain(20, &first_leaf));
    for (right_leaf, valid) in [(&first_leaf, true), (&other_leaf, false)] {
        let right = api(&chain(21, right_leaf));
        let dependencies: Vec<_> = left.functions().chain(right.functions()).cloned().collect();
        let consumer = fixture::fixture(
            30,
            &[CScalarType::I32; 2],
            &dependencies,
            &[Some(0), Some(1)],
        );
        let linked = fixture::linked(&consumer);
        let result = certify_resolved_package(&CDialect, linked);
        assert_eq!(result.is_ok(), valid);
        if !valid {
            let errors = match result {
                Err(errors) => errors,
                Ok(_) => unreachable!(),
            };
            assert!(errors.iter().any(|error| {
                error
                    .message
                    .contains("different certificates for one crate")
            }));
        }
    }
}

#[test]
fn consumer_identity_cannot_reappear_through_a_transitive_dependency() {
    let leaf = fixture::api(10, &[CScalarType::I32]);
    let middle = api(&chain(20, &leaf));
    for call in [None, Some(0)] {
        let consumer = fixture::configured(
            fixture::PackageIdentity {
                crate_id: 10,
                file_stem: "polyrust_reused".into(),
                definition_base: 100,
            },
            &[CScalarType::I32],
            &functions(&middle),
            &[call],
        );
        let linked = fixture::linked(&consumer);
        let errors = match certify_resolved_package(&CDialect, linked) {
            Err(errors) => errors,
            Ok(_) => panic!("transitive source crate identity was reused"),
        };
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("transitive dependency closure"))
        );
    }
}

#[test]
fn complete_transitive_export_and_header_collisions_reject_across_branches() {
    for header_conflict in [false, true] {
        let first = if header_conflict {
            fixture::configured(
                fixture::PackageIdentity {
                    crate_id: 10,
                    file_stem: "a/polyrust_shared".into(),
                    definition_base: 10,
                },
                &[CScalarType::I32],
                &[],
                &[None],
            )
        } else {
            fixture::named(10, &["poly_duplicate_export"], &[], &[None])
        };
        let second = if header_conflict {
            fixture::configured(
                fixture::PackageIdentity {
                    crate_id: 11,
                    file_stem: "b/polyrust_shared".into(),
                    definition_base: 10,
                },
                &[CScalarType::I32],
                &[],
                &[None],
            )
        } else {
            fixture::named(11, &["poly_duplicate_export"], &[], &[None])
        };
        let left = api(&chain(20, &api(&first)));
        let right = api(&chain(21, &api(&second)));
        let dependencies: Vec<_> = left.functions().chain(right.functions()).cloned().collect();
        let consumer = fixture::fixture(
            30,
            &[CScalarType::I32; 2],
            &dependencies,
            &[Some(0), Some(1)],
        );
        let errors = match certify_resolved_package(&CDialect, fixture::linked(&consumer)) {
            Err(errors) => errors,
            Ok(_) => panic!("transitive namespace collision was accepted"),
        };
        let expected = if header_conflict {
            "public header names collide"
        } else {
            "public symbols collide"
        };
        assert!(errors.iter().any(|error| error.message.contains(expected)));
    }
}

#[test]
fn actual_dependency_chain_boundary_rejects_the_next_live_frame() {
    let mut dependency = fixture::api(1000, &[CScalarType::I32]);
    let limit = resources::policy::CResourceKind::NativeFrameBytes.limit();
    for count in 1..=512 {
        let input = chain(1000 + count, &dependency);
        let linked = fixture::linked(&input);
        let measured = resources::measure_package(&linked).unwrap();
        let previous = dependency.functions().next().unwrap().stack_bound_bytes();
        assert_eq!(
            measured.total.frame_bound,
            measured.total.function_frames[&input.functions[0]] + previous
        );
        match certify_resolved_package(&CDialect, linked) {
            Ok(certificate) => {
                assert!(measured.total.frame_bound <= limit);
                dependency = CDependencyApi::from_certificate(certificate).unwrap();
            }
            Err(errors) => {
                assert!(previous <= limit && measured.total.frame_bound > limit);
                assert!(
                    errors
                        .iter()
                        .any(|error| error.message.contains("NativeFrameBytes"))
                );
                eprintln!(
                    "C separate dependency path: {count} admitted packages before next frame exceeds {limit} bytes"
                );
                return;
            }
        }
    }
    panic!("dependency calls did not accumulate their live stack cost");
}
