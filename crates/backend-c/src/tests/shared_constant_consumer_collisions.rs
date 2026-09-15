//! A value import reserves all names in its producer and transitive closure.
use super::{
    CDependencyApi, CDialect,
    constant_consumer_fixture::{Usage, fixture, named, producer},
    dependency_fixture,
    owned_constant_fixture::Shape,
    project_c_package,
};
use portable_codegen::*;

#[test]
fn unused_value_imports_reserve_unselected_constants_against_owned_functions() {
    let owner = producer(Shape::ConstantsOnly);
    let value = owner.constants().next().unwrap().clone();
    for name in ["i32_max", "poly_i32_max"] {
        for usage in [Usage::Read, Usage::Unused] {
            let input = named(90, std::slice::from_ref(&value), None, usage, Some(name));
            let ast = project_c_package(input.registry, input.files).unwrap();
            let checked = verify_unresolved_package(&CDialect, ast).unwrap();
            let errors = TargetLinker::new(CDialect).link_ast(&checked).unwrap_err();
            assert!(format!("{errors:?}").contains("complete dependency export"));
        }
    }
}

#[test]
fn value_edges_participate_in_direct_and_transitive_complete_symbol_collisions() {
    let owner = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    for conflicting in [false, true] {
        let name = if conflicting {
            "poly_i32_max"
        } else {
            "poly_unrelated"
        };
        let remote = dependency_fixture::named(70, &["poly_selected", name], &[], &[None, None]);
        let remote = CDependencyApi::from_certificate(
            certify_resolved_package(&CDialect, dependency_fixture::linked(&remote)).unwrap(),
        )
        .unwrap();
        let selected = remote.functions().next().unwrap().clone();
        let bridge = dependency_fixture::named(
            80,
            &["poly_bridge"],
            std::slice::from_ref(&selected),
            &[Some(0)],
        );
        let bridge = CDependencyApi::from_certificate(
            certify_resolved_package(&CDialect, dependency_fixture::linked(&bridge)).unwrap(),
        )
        .unwrap();
        for transitive in [false, true] {
            let call = if transitive {
                bridge.functions().next().unwrap().clone()
            } else {
                selected.clone()
            };
            for usage in [Usage::Read, Usage::Unused] {
                let input = fixture(90, &values, Some(call.clone()), usage);
                let ast = project_c_package(input.registry.clone(), input.files.clone()).unwrap();
                let checked = verify_unresolved_package(&CDialect, ast).unwrap();
                let result = TargetLinker::new(CDialect).link_ast(&checked);
                if conflicting && !transitive {
                    assert!(
                        format!("{:?}", result.unwrap_err()).contains("complete dependency export")
                    );
                } else if conflicting {
                    let errors = certify_resolved_package(&CDialect, result.unwrap()).unwrap_err();
                    assert!(
                        format!("{errors:?}")
                            .contains("transitive dependency complete public symbols collide")
                    );
                } else {
                    assert!(certify_resolved_package(&CDialect, result.unwrap()).is_ok());
                }
            }
        }
    }
}
