//! Function imports reserve every public constant from their producer.
use super::{
    CDependencyApi, CDependencyFunction, CDialect,
    constant_producer_tests::certify,
    dependency_fixture,
    owned_constant_dependency_fixture::with_call,
    owned_constant_fixture::{self, Shape},
    owned_constant_tests::linked,
    project_c_package,
};
use crate::ast::{CLiteral, CSignedLiteral};
use portable_codegen::{TargetLinker, certify_resolved_package, verify_unresolved_package};

pub(super) fn producer() -> CDependencyApi {
    CDependencyApi::from_certificate(certify(&owned_constant_fixture::fixture(Shape::Mixed)))
        .unwrap()
}

fn getter(api: &CDependencyApi) -> CDependencyFunction {
    api.functions()
        .find(|function| function.function().key().name.as_str() == "read_computed_value")
        .unwrap()
        .clone()
}

pub(super) fn middle(producer: &CDependencyApi) -> CDependencyApi {
    // Deliberately retain an unused registration: its complete linked object
    // surface must still be authenticated, not just the currently executed call.
    let fixture = dependency_fixture::named(80, &["poly_middle"], &[getter(producer)], &[None]);
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, dependency_fixture::linked(&fixture)).unwrap(),
    )
    .unwrap()
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Distance {
    Direct,
    Transitive,
}

pub(super) fn consumer(
    producer: &CDependencyApi,
    middle: &CDependencyApi,
    distance: Distance,
    name: &str,
    call: bool,
) -> owned_constant_fixture::Fixture {
    let (dependency, arguments) = match distance {
        Distance::Direct => (getter(producer), vec![]),
        Distance::Transitive => (
            middle.functions().next().unwrap().clone(),
            vec![CLiteral::Signed(CSignedLiteral::I32(42))],
        ),
    };
    with_call("own_constant", name, dependency, &arguments, call)
}

#[test]
fn function_imports_do_not_omit_unselected_direct_or_transitive_constant_exports() {
    let producer = producer();
    let middle = middle(&producer);
    for name in ["false_value", "poly_false_value"] {
        for call in [false, true] {
            let fixture = consumer(&producer, &middle, Distance::Direct, name, call);
            let projected = project_c_package(fixture.registry, fixture.files).unwrap();
            let checked = verify_unresolved_package(&CDialect, projected).unwrap();
            let errors = TargetLinker::new(CDialect).link_ast(&checked).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|error| error.message.contains("owned C binding collides"))
            );

            let fixture = consumer(&producer, &middle, Distance::Transitive, name, call);
            let errors = certify_resolved_package(&CDialect, linked(&fixture)).unwrap_err();
            assert!(errors.iter().any(|error| {
                error
                    .message
                    .contains("transitive dependency complete public symbols collide")
            }));
        }
    }
    for distance in [Distance::Direct, Distance::Transitive] {
        let fixture = consumer(&producer, &middle, distance, "consumer_read", true);
        certify_resolved_package(&CDialect, linked(&fixture)).unwrap();
    }
}
