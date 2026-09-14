//! Pure graph contracts do not need rustc internals, backend C, or a filesystem.
#[path = "crate_graph_arguments.rs"]
mod arguments;
use portable_rustc_configuration::{
    Configuration, Mode,
    graph::{CrateDescription, CrateGraph, InputMapping},
};

fn input(physical: &str, logical: &str) -> InputMapping {
    InputMapping::new(physical, logical).unwrap()
}

fn node(key: &str) -> CrateDescription {
    CrateDescription::new(
        "same_name",
        key,
        input(&format!("/{key}/lib.rs"), "src/lib.rs"),
        &format!("/{key}/libsame_name.rmeta"),
    )
    .unwrap()
}

fn keys(graph: &CrateGraph) -> Vec<&str> {
    graph.crates().iter().map(CrateDescription::key).collect()
}

#[test]
fn deterministic_diamond_order_preserves_aliases_and_separate_owners() {
    let leaf = node("leaf");
    let left = node("left").with_dependency("shared", "leaf").unwrap();
    let right = node("right").with_dependency("renamed", "leaf").unwrap();
    let root = node("root")
        .with_dependency("r", "right")
        .unwrap()
        .with_dependency("l", "left")
        .unwrap()
        .with_dependency("second_l", "left")
        .unwrap();
    let first = CrateGraph::new(
        "root",
        vec![root.clone(), right.clone(), leaf.clone(), left.clone()],
    )
    .unwrap();
    let second = CrateGraph::new("root", vec![left, leaf, right, root]).unwrap();
    assert_eq!(first, second);
    assert_eq!(keys(&first), ["leaf", "left", "right", "root"]);
    assert_eq!(first.root_key(), "root");
    assert_eq!(
        first.crates()[3].dependencies().collect::<Vec<_>>(),
        [("l", "left"), ("r", "right"), ("second_l", "left")]
    );
    assert!(first.crates().iter().all(|node| node.name() == "same_name"));
    assert_eq!(first.rooted_at("root").unwrap(), first);
    let left = first.rooted_at("left").unwrap();
    assert_eq!(keys(&left), ["leaf", "left"]);
    assert_eq!(left.root_key(), "left");
    assert_eq!(left.crates()[1], first.crates()[1]);
    assert_eq!(keys(&first.rooted_at("right").unwrap()), ["leaf", "right"]);
    assert_eq!(keys(&left.rooted_at("leaf").unwrap()), ["leaf"]);
    assert!(
        first
            .rooted_at("missing")
            .unwrap_err()
            .contains("not declared")
    );
    assert!(first.rooted_at("bad key").is_err());
}

#[test]
fn missing_duplicate_unreachable_and_cyclic_graphs_reject() {
    for (root, descriptions, expected) in [
        ("root", vec![], "1..1024"),
        ("missing", vec![node("root")], "root is not declared"),
        (
            "root",
            vec![node("root"), node("root")],
            "metadata artifact",
        ),
        ("root", vec![node("root"), node("unused")], "unreachable"),
        (
            "root",
            vec![node("root").with_dependency("missing", "missing").unwrap()],
            "dependency is not declared",
        ),
        (
            "root",
            vec![node("root").with_dependency("self_edge", "root").unwrap()],
            "cycle",
        ),
        (
            "root",
            vec![
                node("root").with_dependency("other", "other").unwrap(),
                node("other").with_dependency("back", "root").unwrap(),
            ],
            "cycle",
        ),
    ] {
        let error = CrateGraph::new(root, descriptions).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
    let duplicate_key = CrateDescription::new(
        "different",
        "root",
        input("/other.rs", "other.rs"),
        "/other.rmeta",
    )
    .unwrap();
    assert!(
        CrateGraph::new("root", vec![node("root"), duplicate_key])
            .unwrap_err()
            .contains("duplicate crate graph key")
    );
}

#[test]
fn crate_count_has_real_exact_limit_and_one_over_tests() {
    let mut descriptions = Vec::new();
    for index in 0..1024 {
        let mut value = node(&format!("n{index:04}"));
        if index != 0 {
            value = value
                .with_dependency("previous", &format!("n{:04}", index - 1))
                .unwrap();
        }
        descriptions.push(value);
    }
    let graph = CrateGraph::new("n1023", descriptions.clone()).unwrap();
    assert_eq!(graph.crates().len(), 1024);
    assert_eq!(graph.crates()[0].key(), "n0000");
    assert_eq!(graph.rooted_at("n1023").unwrap(), graph);
    assert_eq!(graph.rooted_at("n0511").unwrap().crates().len(), 512);
    descriptions.push(node("n1024").with_dependency("previous", "n1023").unwrap());
    assert!(
        CrateGraph::new("n1024", descriptions)
            .unwrap_err()
            .contains("1..1024")
    );
}

#[test]
fn input_and_alias_builders_reject_duplicates_without_accepting_flags() {
    assert!(
        node("root")
            .with_dependency("alias", "one")
            .unwrap()
            .with_dependency("alias", "two")
            .is_err()
    );
    assert!(node("root").with_dependency("--extern", "one").is_err());
    assert!(node("root").with_dependency("alias", "bad key").is_err());
    assert!(
        node("root")
            .with_input(input("/other", "src/lib.rs"))
            .is_err()
    );
    assert!(
        node("root")
            .with_input(input("/root/lib.rs", "other.rs"))
            .is_err()
    );
    let description = node("root")
        .with_input(input("/root/docs.md", "docs/api.md"))
        .unwrap();
    assert_eq!(description.inputs().len(), 2);
    assert_eq!(description.root().logical(), "src/lib.rs");
    assert_eq!(description.root().physical(), "/root/lib.rs");
    assert_eq!(description.metadata(), "/root/libsame_name.rmeta");
    let configuration = description.configuration();
    assert_eq!(
        configuration,
        Configuration::package("same_name", "root").unwrap()
    );
    assert_eq!(configuration.mode(), Mode::PublicPackage);
    assert!(configuration.declared_inputs().is_empty());
    let arguments = configuration.compiler_arguments("/root/lib.rs", "/compiler");
    assert!(arguments.iter().any(|arg| arg == "-Funsafe-code"));
    assert_eq!(arguments.last().unwrap(), "-Cmetadata=root");
}

#[test]
fn logical_names_and_physical_descriptor_byte_limits_are_checked() {
    for logical in [
        "",
        "/root.rs",
        "../root.rs",
        "src/./root.rs",
        "src//root.rs",
        "src/",
        "src\\root.rs",
        "C:/root.rs",
        "a=b.rs",
        "café.rs",
    ] {
        assert!(InputMapping::new("/real.rs", logical).is_err(), "{logical}");
    }
    for physical in ["", "/a=b.rs", "/a\nb.rs", "/a\0b.rs"] {
        assert!(InputMapping::new(physical, "root.rs").is_err());
    }
    assert!(InputMapping::new(&"x".repeat(4096), &"x".repeat(4096)).is_ok());
    assert!(InputMapping::new(&"x".repeat(4097), "root.rs").is_err());
    assert!(InputMapping::new("/root.rs", &"x".repeat(4097)).is_err());
    assert!(
        CrateDescription::new("name", "key", input("/root.rs", "root.txt"), "/lib.rmeta").is_err()
    );
    assert!(
        CrateDescription::new("name", "key", input("/root.rs", "root.rs"), "/lib.rlib").is_err()
    );
}

#[test]
fn metadata_cannot_overlap_another_artifact_or_declared_input() {
    let shared = CrateDescription::new(
        "other",
        "other",
        input("/other.rs", "other.rs"),
        node("root").metadata(),
    )
    .unwrap();
    let root = node("root").with_dependency("other", "other").unwrap();
    assert!(
        CrateGraph::new("root", vec![root.clone(), shared])
            .unwrap_err()
            .contains("metadata artifact path")
    );
    let overlap = node("other")
        .with_input(input(root.metadata(), "include.bin"))
        .unwrap();
    assert!(
        CrateGraph::new("root", vec![root, overlap])
            .unwrap_err()
            .contains("overlaps a declared source")
    );
}

#[test]
fn per_crate_input_count_has_an_actual_boundary() {
    let mut description = node("root");
    for index in 1..4096 {
        description = description
            .with_input(input(
                &format!("/docs/{index}.md"),
                &format!("docs/{index}.md"),
            ))
            .unwrap();
    }
    assert_eq!(description.inputs().len(), 4096);
    assert!(
        description
            .with_input(input("/docs/extra.md", "docs/extra.md"))
            .is_err()
    );
}

#[test]
fn aggregate_path_budget_rejects_individually_valid_descriptors() {
    let mut description = node("root");
    for index in 1..2049 {
        description = description
            .with_input(input(
                &format!("{}{:04}", "a".repeat(4092), index),
                &format!("d{index}.md"),
            ))
            .unwrap();
    }
    assert!(
        CrateGraph::new("root", vec![description])
            .unwrap_err()
            .contains("budget exceeded")
    );
}
