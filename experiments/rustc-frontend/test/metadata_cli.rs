//! Response files preserve the graph grammar beyond OS argv limits.
#[path = "../src/metadata_cli.rs"]
mod metadata_cli;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn path() -> PathBuf {
    PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join(format!(
        "metadata-args-{}",
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn record() -> Vec<String> {
    [
        "--root",
        "key",
        "--crate",
        "fixture",
        "key",
        "/source with spaces.rs",
        "lib.rs",
        "/output.rmeta",
    ]
    .map(str::to_owned)
    .to_vec()
}

fn response(path: &std::path::Path) -> String {
    format!("@{}", path.display())
}

#[test]
fn verbatim_response_and_direct_records_have_identical_typed_graphs() {
    let path = path();
    let arguments = record();
    fs::write(&path, arguments.join("\n") + "\n").unwrap();
    assert_eq!(
        metadata_cli::parse(&[response(&path)]).unwrap(),
        metadata_cli::parse(&arguments).unwrap()
    );
}

#[test]
fn a_valid_multi_megabyte_graph_does_not_require_multi_megabyte_argv() {
    let path = path();
    let mut arguments = record();
    for index in 0..800 {
        arguments.extend([
            "--input".into(),
            format!("/physical/{index}"),
            format!("docs/{index}/{}", "a".repeat(3000)),
        ]);
    }
    let contents = arguments.join("\n") + "\n";
    assert!(contents.len() > 2 * 1024 * 1024);
    fs::write(&path, contents).unwrap();
    let graph = metadata_cli::parse(&[response(&path)]).unwrap();
    assert_eq!(graph.crates()[0].inputs().len(), 801);
}

#[test]
fn malformed_nested_mixed_and_oversized_response_files_reject() {
    let path = path();
    for text in [
        b"".as_slice(),
        b"@nested",
        b"--root\nkey\n--extern\nambient",
        b"\xff",
        b"--root\r\nkey\r\n",
    ] {
        fs::write(&path, text).unwrap();
        assert!(metadata_cli::parse(&[response(&path)]).is_err());
    }
    fs::write(&path, record().join("\n")).unwrap();
    assert!(metadata_cli::parse(&[response(&path), "--root".into(), "key".into()]).is_err());
    fs::File::create(&path)
        .unwrap()
        .set_len(32 * 1024 * 1024 + 1)
        .unwrap();
    assert!(
        metadata_cli::parse(&[response(&path)])
            .unwrap_err()
            .contains("32 MiB")
    );
    assert!(metadata_cli::parse(&[response(&path.with_extension("missing"))]).is_err());
    assert!(metadata_cli::parse(&["@".into()]).is_err());
    fs::write(
        &path,
        "\n".repeat(portable_rustc_configuration::graph::CrateGraph::MAX_ARGUMENTS + 2),
    )
    .unwrap();
    assert!(
        metadata_cli::parse(&[response(&path)])
            .unwrap_err()
            .contains("argument count")
    );
}
