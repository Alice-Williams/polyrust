//! Whole-header namespace and transitive certificate composition controls.
use super::{dependency_fixture as scalar, nominal_fixture as fixture, nominal_producer::producer};
use crate::{ast::*, dialect::*};
use portable_codegen::*;

fn scalar_consumer(
    crate_id: u64,
    owners: &[&CDependencyApi],
) -> Result<RenderReadyPackage<CDialect>, String> {
    let dependencies: Vec<_> = owners
        .iter()
        .map(|api| api.functions().next().unwrap().clone())
        .collect();
    let input = scalar::fixture(
        crate_id,
        &vec![CScalarType::I32; dependencies.len()],
        &dependencies,
        &(0..dependencies.len()).map(Some).collect::<Vec<_>>(),
    );
    let ast = project_c_package(input.registry, input.files).map_err(|e| format!("{e:?}"))?;
    let checked = verify_unresolved_package(&CDialect, ast).map_err(|e| format!("{e:?}"))?;
    let linked = TargetLinker::new(CDialect)
        .link_ast(&checked)
        .map_err(|e| format!("{e:?}"))?;
    certify_resolved_package(&CDialect, linked).map_err(|e| format!("{e:?}"))
}

#[test]
fn scalar_imports_reconcile_even_unused_header_tags_directly_and_transitively() {
    let a = producer(90, &["first", "collision"], "first_api");
    let b = producer(91, &["second", "collision"], "second_api");
    let error = scalar_consumer(92, &[&a, &b]).unwrap_err();
    assert!(error.contains("complete public tags collide"), "{error}");
    let left = CDependencyApi::from_certificate(scalar_consumer(93, &[&a]).unwrap()).unwrap();
    let right = CDependencyApi::from_certificate(scalar_consumer(94, &[&b]).unwrap()).unwrap();
    let error = scalar_consumer(95, &[&left, &right]).unwrap_err();
    assert!(error.contains("complete public tags collide"), "{error}");
}

#[test]
fn public_tags_and_ordinary_functions_remain_distinct_namespaces() {
    let a = producer(96, &["shared"], "first_api");
    let b = producer(97, &["second"], "shared");
    let package = scalar_consumer(98, &[&a, &b]).unwrap();
    render_certified_package(&CStructuralRenderer, &package).unwrap();
}

#[test]
fn type_only_imports_cannot_hide_conflicting_transitive_authorities_or_source_cycles() {
    let a = producer(100, &["first"], "first_api");
    let other = producer(100, &["first"], "first_api");
    let left_proof = a.structs().next().unwrap().clone();
    let right_proof = other.structs().next().unwrap().clone();
    // Scalar calls hide both foreign types from their own exported signatures.
    let left = CDependencyApi::from_certificate(
        fixture::certify(fixture::fixture(
            101,
            std::slice::from_ref(&left_proof),
            &[fixture::Operation::Call(
                a.functions().next().unwrap().clone(),
            )],
        ))
        .unwrap(),
    )
    .unwrap();
    let right = CDependencyApi::from_certificate(
        fixture::certify(fixture::fixture(
            102,
            &[right_proof],
            &[fixture::Operation::Call(
                other.functions().next().unwrap().clone(),
            )],
        ))
        .unwrap(),
    )
    .unwrap();
    let error = scalar_consumer(103, &[&left, &right]).unwrap_err();
    assert!(
        error.contains("different certificates for one crate"),
        "{error}"
    );
    let error = fixture::certify(fixture::fixture(
        100,
        &[left_proof],
        &[fixture::Operation::Copy],
    ))
    .unwrap_err();
    assert!(
        error.contains("consumer crate identity overlaps"),
        "{error}"
    );
    let error = scalar_consumer(100, &[&left]).unwrap_err();
    assert!(error.contains("transitive dependency closure"), "{error}");
}

#[test]
fn nominal_header_collisions_reject_registration_in_both_orders_without_partial_updates() {
    let owner = producer(104, &["result"], "api");
    let proof = owner.structs().next().unwrap().clone();
    for file_first in [true, false] {
        let mut registry = CRegistry::new();
        let file = CFileKey {
            path: RelativeOutputPath::new("nested/polyrust_producer_104.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        };
        if file_first {
            registry.register_file(file).unwrap();
            let before = format!("{registry:?}");
            assert!(registry.import_struct(proof.clone()).is_err());
            assert_eq!(format!("{registry:?}"), before);
        } else {
            registry.import_struct(proof.clone()).unwrap();
            let before = format!("{registry:?}");
            assert!(registry.register_file(file).is_err());
            assert_eq!(format!("{registry:?}"), before);
        }
    }
}
