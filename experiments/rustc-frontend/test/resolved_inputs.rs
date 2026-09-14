//! Linux filesystem boundary tests; all writes stay under Bazel's test directory.
use portable_rustc_configuration::graph::{CrateDescription, InputMapping, ResolvedInputs};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn directory() -> PathBuf {
    let base = PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel TEST_TMPDIR"));
    let path = base.join(format!(
        "resolved-inputs-{}",
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn descriptor(root: &Path) -> CrateDescription {
    CrateDescription::new(
        "fixture",
        "test.key",
        InputMapping::new(root.to_str().unwrap(), "src/lib.rs").unwrap(),
        "/unused/libfixture.rmeta",
    )
    .unwrap()
}

#[test]
fn canonical_inputs_produce_declared_read_and_exact_logical_mapping_arguments() {
    let work = directory();
    let root = work.join("root with spaces.rs");
    let doc = work.join("api.md");
    fs::write(&root, "pub fn identity(value: i32) -> i32 { value }").unwrap();
    fs::write(&doc, "Documentation.").unwrap();
    let description = descriptor(&root)
        .with_input(InputMapping::new(doc.to_str().unwrap(), "docs/api.md").unwrap())
        .unwrap();
    let resolved = ResolvedInputs::load(&description).unwrap();
    assert_eq!(
        resolved.root(),
        root.canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(resolved.mappings().len(), 2);
    assert_eq!(
        resolved.declared_input_arguments(),
        ["--input", doc.to_str().unwrap()]
    );
    let arguments = resolved
        .metadata_arguments("/pinned/sysroot", "/staging/output.rmeta")
        .unwrap();
    for expected in [
        "--crate-name=fixture",
        "-Cmetadata=test.key",
        "-Funsafe-code",
        "--emit=metadata",
        "-Cpanic=abort",
        "-Copt-level=0",
        "--edition=2024",
    ] {
        assert!(arguments.iter().any(|arg| arg == expected), "{expected}");
    }
    let mappings: Vec<_> = arguments
        .iter()
        .filter(|arg| arg.starts_with("--remap-path-prefix="))
        .collect();
    let expected = [
        &format!(
            "--remap-path-prefix={}=/polyrust/build",
            std::env::current_dir().unwrap().display()
        ),
        &format!(
            "--remap-path-prefix={}=/polyrust/inputs/fixture/docs/api.md",
            doc.display()
        ),
        &format!(
            "--remap-path-prefix={}=/polyrust/inputs/fixture/src/lib.rs",
            root.display()
        ),
    ];
    assert_eq!(mappings.len(), expected.len());
    for mapping in expected {
        assert!(mappings.contains(&mapping), "missing mapping: {mapping}");
        let (physical, logical) = mapping
            .strip_prefix("--remap-path-prefix=")
            .unwrap()
            .split_once('=')
            .unwrap();
        let actual = mappings
            .iter()
            .rev()
            .find_map(|rule| {
                let (prefix, replacement) = rule
                    .strip_prefix("--remap-path-prefix=")
                    .unwrap()
                    .split_once('=')
                    .unwrap();
                physical
                    .strip_prefix(prefix)
                    .map(|suffix| format!("{replacement}{suffix}"))
            })
            .unwrap();
        assert_eq!(
            actual, logical,
            "last matching textual remap for {physical}"
        );
    }
    assert_eq!(fs::read_to_string(doc).unwrap(), "Documentation.");
}

#[test]
fn missing_and_non_file_inputs_reject_without_creating_anything() {
    let work = directory();
    let missing = work.join("missing.rs");
    assert!(ResolvedInputs::load(&descriptor(&missing)).is_err());
    assert!(!missing.exists());
    assert!(
        ResolvedInputs::load(&descriptor(&work))
            .unwrap_err()
            .contains("not a regular file")
    );
    assert_eq!(fs::read_dir(work).unwrap().count(), 0);
}

#[test]
fn overlapping_textual_prefixes_emit_more_specific_mappings_last() {
    let work = directory();
    let root = work.join("prefix.rs");
    let doc = work.join("prefix");
    fs::write(&root, "").unwrap();
    fs::write(&doc, "").unwrap();
    let description = CrateDescription::new(
        "fixture",
        "test.key",
        InputMapping::new(root.to_str().unwrap(), "a.rs").unwrap(),
        "/unused/libfixture.rmeta",
    )
    .unwrap()
    .with_input(InputMapping::new(doc.to_str().unwrap(), "z.md").unwrap())
    .unwrap();
    let arguments = ResolvedInputs::load(&description)
        .unwrap()
        .metadata_arguments("/pinned/sysroot", "/staging/output.rmeta")
        .unwrap();
    let mappings: Vec<_> = arguments
        .iter()
        .filter(|arg| arg.starts_with("--remap-path-prefix="))
        .collect();
    let shorter = mappings
        .iter()
        .position(|rule| rule.ends_with("/z.md"))
        .unwrap();
    let longer = mappings
        .iter()
        .position(|rule| rule.ends_with("/a.rs"))
        .unwrap();
    assert!(shorter < longer);
}

#[cfg(unix)]
#[test]
fn canonical_aliases_and_invalid_resolved_delimiters_reject() {
    let work = directory();
    let root = work.join("lib.rs");
    let alias = work.join("alias.rs");
    fs::write(&root, "pub fn identity(value: i32) -> i32 { value }").unwrap();
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    let description = descriptor(&root)
        .with_input(InputMapping::new(alias.to_str().unwrap(), "alias.rs").unwrap())
        .unwrap();
    assert!(
        ResolvedInputs::load(&description)
            .unwrap_err()
            .contains("multiple logical mappings")
    );
    let bad = work.join("bad=target.rs");
    let linked = work.join("linked.rs");
    fs::write(&bad, "").unwrap();
    std::os::unix::fs::symlink(&bad, &linked).unwrap();
    assert!(
        ResolvedInputs::load(&descriptor(&linked))
            .unwrap_err()
            .contains("without '='")
    );
}

#[cfg(unix)]
#[test]
fn a_single_symlink_resolves_to_its_declared_logical_identity() {
    let work = directory();
    let root = work.join("real.rs");
    let alias = work.join("alias.rs");
    fs::write(&root, "pub fn identity(value: i32) -> i32 { value }").unwrap();
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    let direct = ResolvedInputs::load(&descriptor(&root)).unwrap();
    let linked = ResolvedInputs::load(&descriptor(&alias)).unwrap();
    assert_eq!(direct, linked);
}

#[cfg(unix)]
#[test]
fn hardlinked_source_inputs_cannot_claim_distinct_logical_identities() {
    let work = directory();
    let root = work.join("root.rs");
    let duplicate = work.join("duplicate.rs");
    fs::write(&root, "").unwrap();
    fs::hard_link(&root, &duplicate).unwrap();
    let description = descriptor(&root)
        .with_input(InputMapping::new(duplicate.to_str().unwrap(), "duplicate.rs").unwrap())
        .unwrap();
    assert!(
        ResolvedInputs::load(&description)
            .unwrap_err()
            .contains("inode alias")
    );
}
