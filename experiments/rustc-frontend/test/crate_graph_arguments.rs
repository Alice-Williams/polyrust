//! Unknown graph input can only enter through the closed descriptor protocol.
use portable_rustc_configuration::graph::CrateGraph;

fn parse(arguments: &[&str]) -> Result<CrateGraph, String> {
    CrateGraph::parse(
        &arguments
            .iter()
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>(),
    )
}

#[test]
fn closed_records_resolve_forward_edges_and_order_inputs() {
    let graph = parse(&[
        "--root",
        "root",
        "--crate",
        "consumer",
        "root",
        "/root/lib.rs",
        "src/lib.rs",
        "/root/libconsumer.rmeta",
        "--input",
        "/root/docs.md",
        "docs/api.md",
        "--dependency",
        "renamed",
        "leaf",
        "--crate",
        "dependency",
        "leaf",
        "/leaf/lib.rs",
        "src/lib.rs",
        "/leaf/libdependency.rmeta",
    ])
    .unwrap();
    assert_eq!(graph.root_key(), "root");
    assert_eq!(
        graph.crates().iter().map(|c| c.key()).collect::<Vec<_>>(),
        ["leaf", "root"]
    );
    assert_eq!(
        graph.crates()[1].dependencies().collect::<Vec<_>>(),
        [("renamed", "leaf")]
    );
    assert_eq!(
        graph.crates()[1]
            .inputs()
            .map(|i| i.logical())
            .collect::<Vec<_>>(),
        ["docs/api.md", "src/lib.rs"]
    );
}

#[test]
fn incomplete_misplaced_duplicate_and_unknown_records_are_rejected() {
    for arguments in [
        vec![],
        vec!["--root"],
        vec!["--crate", "name"],
        vec!["--root", "root"],
        vec!["--root", "root", "--root", "root"],
        vec!["--root", "root", "--input", "/a", "a.rs"],
        vec!["--root", "root", "--dependency", "alias", "key"],
    ] {
        assert!(parse(&arguments).is_err(), "{arguments:?}");
    }
    let node = [
        "--root",
        "root",
        "--crate",
        "name",
        "root",
        "/src.rs",
        "src.rs",
        "/lib.rmeta",
    ];
    for length in 2..node.len() {
        assert!(parse(&node[..length]).is_err());
    }
    for trailing in [
        vec!["--input"],
        vec!["--input", "/doc.md"],
        vec!["--dependency"],
        vec!["--dependency", "alias"],
        vec!["--input", "/src.rs", "src.rs"],
        vec![
            "--dependency",
            "alias",
            "root",
            "--dependency",
            "alias",
            "root",
        ],
        vec!["--extern", "alias=/ambient.rmeta"],
        vec!["--sysroot", "/other"],
        vec!["-Aunsafe-code"],
        vec!["--emit", "link"],
        vec!["-Ldependency=/ambient"],
        vec!["--remap-path-prefix=/source=/other"],
        vec!["--crate-name", "other"],
    ] {
        let mut arguments = node.to_vec();
        arguments.extend(trailing);
        assert!(parse(&arguments).is_err(), "{arguments:?}");
    }
}

#[test]
fn parser_retains_graph_semantic_checks() {
    let node = [
        "--root",
        "root",
        "--crate",
        "name",
        "root",
        "/src.rs",
        "src.rs",
        "/lib.rmeta",
    ];
    for (target, expected) in [("absent", "not declared"), ("root", "cycle")] {
        let mut arguments = node.to_vec();
        arguments.extend(["--dependency", "alias", target]);
        assert!(parse(&arguments).unwrap_err().contains(expected));
    }
    let mut duplicate = node.to_vec();
    duplicate.extend([
        "--crate",
        "other",
        "root",
        "/other.rs",
        "other.rs",
        "/other.rmeta",
    ]);
    assert!(
        parse(&duplicate)
            .unwrap_err()
            .contains("duplicate crate graph key")
    );
}
