//! Owned constants participate in projection, reservation and owner mutation checks.
use crate::{Owner, PreparedBundle, constant_fixture, fixture, projection};
use portable_backend_java::ast::JavaLiteral;

#[test]
fn constant_descriptions_are_exact_and_cannot_be_replaced() {
    let api = constant_fixture::owner(
        7,
        " Constant docs.",
        JavaLiteral::I64(9_007_199_254_740_993),
    );
    let owner = Owner {
        key: "constant",
        api: &api,
    };
    let mut manifest = projection::project(owner).unwrap();
    manifest.verify_owner().unwrap();
    let saved = manifest.declarations.clone();
    manifest.declarations.clear();
    assert!(manifest.verify_owner().is_err());
    manifest.declarations = saved.clone();
    manifest.declarations.push(saved[0]);
    assert!(manifest.verify_owner().is_err());
    for wrong in [
        JavaLiteral::I64(17),
        JavaLiteral::I32(17),
        JavaLiteral::Boolean(true),
    ] {
        let other = constant_fixture::owner(7, " Constant docs.", wrong);
        let mut altered = projection::project(owner).unwrap();
        altered.declarations = other.source_descriptions().unwrap();
        assert!(altered.verify_owner().is_err());
    }
    let unrelated = fixture::owner(7, " Function docs.");
    manifest.declarations = unrelated.source_descriptions().unwrap();
    assert!(manifest.verify_owner().is_err());
}

#[test]
fn constants_only_mixed_owner_graphs_preserve_values_docs_and_versions() {
    let constant = constant_fixture::owner(
        7,
        " Constant docs.",
        JavaLiteral::I64(9_007_199_254_740_993),
    );
    let function = fixture::owner(8, " Function docs.");
    let owners = [
        Owner {
            key: "constant",
            api: &constant,
        },
        Owner {
            key: "function",
            api: &function,
        },
    ];
    let prepared = PreparedBundle::new(constant.root(), &owners).unwrap();
    let output = prepared.render().unwrap();
    assert_eq!(output.owner_count(), 2);
    assert_eq!(output.files().len(), 5);
    assert!(
        output
            .files()
            .iter()
            .map(|(_, text)| text.len() as u64)
            .sum::<u64>()
            <= prepared.reserved_bytes()
    );
    let json = &output
        .files()
        .iter()
        .find(|(p, _)| p == "polyrust_0000000000000007.api.json")
        .unwrap()
        .1;
    assert!(json.contains("\"schema_version\":2"));
    assert!(json.contains("\"kind\":\"constant\""));
    assert!(json.contains("\"scalar\":\"i64\",\"readonly\":true,\"value\":\"9007199254740993\""));
    assert!(json.contains("Constant docs."));
    let old = &output
        .files()
        .iter()
        .find(|(p, _)| p == "polyrust_0000000000000008.api.json")
        .unwrap()
        .1;
    assert!(old.contains("\"schema_version\":1"));
    assert!(!old.contains("\"kind\":\"constant\""));
    let reversed = [owners[1], owners[0]];
    assert_eq!(
        output.files(),
        PreparedBundle::new(constant.root(), &reversed)
            .unwrap()
            .render()
            .unwrap()
            .files()
    );
    assert!(PreparedBundle::new(constant.root(), &owners[1..]).is_err());
    assert!(PreparedBundle::new(constant.root(), &[owners[0], owners[0]]).is_err());
}
