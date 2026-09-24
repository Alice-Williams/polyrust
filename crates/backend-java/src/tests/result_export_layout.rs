//! Layout-only evidence, not dependency publication or consumer registration.
use super::*;
use crate::tests::scalar_result_families::fixture::{Fixture, certify};

#[test]
fn result_export_layout_retains_original_identities_paths_and_payload() {
    let fixture = Fixture::new();
    let types = fixture.family.types;
    let other = fixture.other.types;
    let package = certify(fixture.finish());
    let layouts = collect(&package, &[types, other]).unwrap();
    assert_eq!(layouts.len(), 2);
    let layout = &layouts[&types.interface];
    assert_eq!(layout.types, types);
    assert_eq!(layout.payload_name.as_str(), "value");
    for (path, expected) in layout.paths.iter().zip(["Outcome", "Success", "Error"]) {
        assert_eq!(path.package(), crate::ast::JavaPackage::RustCrate(7));
        assert_eq!(path.owners().len(), 1);
        assert_eq!(path.owners()[0].as_str(), "ResultFixture");
        assert_eq!(path.member().as_str(), expected);
    }
    assert_ne!(layout.paths, layouts[&other.interface].paths);
}

#[test]
fn result_export_layout_rejects_repeated_mixed_and_hidden_families() {
    let fixture = Fixture::new();
    let types = fixture.family.types;
    let other = fixture.other.types;
    let package = certify(fixture.finish());
    assert!(
        collect(&package, &[types, types])
            .unwrap_err()
            .contains("overlap or repeat")
    );
    assert!(
        collect(
            &package,
            &[JavaScalarResultTypes {
                error: other.error,
                ..types
            }]
        )
        .is_err()
    );
    let fixture = Fixture::with_facade_visibility(JavaVisibility::Package);
    let types = fixture.family.types;
    let package = certify(fixture.finish());
    JavaScalarResultFamily::from_certificate(package.clone(), types).unwrap();
    assert!(
        collect(&package, &[types])
            .unwrap_err()
            .contains("public enclosing")
    );
}

#[test]
fn result_export_layout_search_and_selection_budgets_have_exact_boundaries() {
    let fixture = Fixture::new();
    let selections = [fixture.family.types, fixture.other.types];
    let package = certify(fixture.finish());
    let JavaFileItem::Type { declaration, .. } = &package.ast().files()[0].items()[0].item else {
        unreachable!()
    };
    // Five methods and six nested types; two selections, six charged scans each.
    assert_eq!(declaration.members.len(), 11);
    assert_eq!(
        collect_with_limits(&package, &selections, 2, 144)
            .unwrap()
            .len(),
        2
    );
    assert!(
        collect_with_limits(&package, &selections, 1, 144)
            .unwrap_err()
            .contains("selection limit")
    );
    assert!(
        collect_with_limits(&package, &selections, 2, 143)
            .unwrap_err()
            .contains("search limit")
    );
    assert!(collect_with_limits(&package, &[], 0, 0).unwrap().is_empty());
}
