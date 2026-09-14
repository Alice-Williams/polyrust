// Rust-source provenance survives the shared target graph without fake Core IDs.
use super::{TestDialect, valid_fixture};
use crate::{
    GeneratedOrigin, RustDeclarationId, RustSourceLocation, RustSourceNode, RustSourceOrigin,
    RustVisibility, verify_target_ast,
};
use std::sync::Arc;

fn metadata() -> RustSourceOrigin {
    let declaration = RustDeclarationId {
        crate_id: 3,
        definition_path_hash: 8,
    };
    RustSourceOrigin {
        declaration,
        node: RustSourceNode::Declaration,
        module: RustDeclarationId {
            definition_path_hash: 2,
            ..declaration
        },
        location: RustSourceLocation {
            file: "src/model.rs".into(),
            line: 7,
            column: 1,
        },
        visibility: RustVisibility::Public,
        externally_reachable: true,
        documentation: vec!["A source declaration.".into()],
        module_ancestors: [].into(),
        crate_exports: Arc::new(crate::RustCrateExports {
            module_ancestries: std::collections::BTreeMap::new(),
            root: declaration,
            modules: Default::default(),
        }),
    }
}

#[test]
fn rust_metadata_survives_type_callable_and_value_registration() {
    let (mut package, _) = valid_fixture();
    let expected = Arc::new(metadata());
    let origin = GeneratedOrigin::RustSource(expected.clone());
    package.types[0].origin = origin.clone();
    package.callables[0].origin = origin.clone();
    package.values[0].origin = origin;
    assert_eq!(verify_target_ast(&package), Ok(()));
    let copied = package.clone();
    for origin in [
        &copied.generated_types().next().unwrap().origin,
        &copied.callables().next().unwrap().origin,
        &copied.values().next().unwrap().origin,
    ] {
        let GeneratedOrigin::RustSource(actual) = origin else {
            panic!("lost source provenance");
        };
        assert_eq!(actual, &expected);
        assert!(Arc::ptr_eq(actual, &expected));
    }
}

#[test]
fn source_identity_uses_values_not_allocation_addresses_or_spelling() {
    let first: GeneratedOrigin<TestDialect> = GeneratedOrigin::RustSource(Arc::new(metadata()));
    let second: GeneratedOrigin<TestDialect> = GeneratedOrigin::RustSource(Arc::new(metadata()));
    assert_eq!(first, second);
    let mut other_crate = metadata();
    other_crate.declaration.crate_id += 1;
    assert_ne!(first, GeneratedOrigin::RustSource(Arc::new(other_crate)));
    let mut body_node = metadata();
    body_node.node = RustSourceNode::Binding(8);
    assert_ne!(first, GeneratedOrigin::RustSource(Arc::new(body_node)));
}
