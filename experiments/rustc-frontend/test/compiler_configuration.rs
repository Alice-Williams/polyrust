//! Pure argument-model tests, independently cached from compiler integration.
use portable_rustc_configuration::{Configuration, Mode};

fn parse(arguments: &[&str]) -> Result<Configuration, String> {
    Configuration::parse(
        &arguments
            .iter()
            .map(|value| (*value).into())
            .collect::<Vec<_>>(),
    )
}

#[test]
fn default_invocations_preserve_fixed_compiler_configuration() {
    for (flags, mode) in [
        (vec![], Mode::Entry),
        (vec!["--package"], Mode::PublicPackage),
    ] {
        let configuration = parse(&flags).unwrap();
        assert_eq!(configuration.mode(), mode);
        assert!(configuration.declared_inputs().is_empty());
        assert_eq!(
            configuration.compiler_arguments("/source.rs", "/compiler"),
            [
                "rustc",
                "/source.rs",
                "--sysroot",
                "/compiler",
                "--crate-type=lib",
                "--crate-name=poly_input",
                "--edition=2024",
                "-Funsafe-code",
                "-Flong-running-const-eval",
                "-Copt-level=0",
                "-Cpanic=abort",
            ]
        );
    }
}

#[test]
fn explicit_identity_is_order_independent_without_forwarding_options() {
    let first = parse(&[
        "--package",
        "--crate-name",
        "example",
        "--crate-key",
        "//pkg:example",
        "--input",
        "docs.md",
    ])
    .unwrap();
    let second = parse(&[
        "--package",
        "--input",
        "docs.md",
        "--crate-key",
        "//pkg:example",
        "--crate-name",
        "example",
    ])
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.declared_inputs(), ["--input", "docs.md"]);
    let arguments = first.compiler_arguments("/source.rs", "/compiler");
    assert_eq!(arguments.len(), 12);
    assert!(arguments.contains(&"--crate-name=example".into()));
    assert_eq!(arguments.last().unwrap(), "-Cmetadata=//pkg:example");
    for required in [
        "-Funsafe-code",
        "-Flong-running-const-eval",
        "-Copt-level=0",
        "-Cpanic=abort",
        "--edition=2024",
        "--crate-type=lib",
    ] {
        assert!(arguments.iter().any(|value| value == required));
    }
}

#[test]
fn identity_byte_limits_have_exact_positive_and_negative_boundaries() {
    let arguments = |name: String, key: String| {
        vec![
            "--package".into(),
            "--crate-name".into(),
            name,
            "--crate-key".into(),
            key,
        ]
    };
    assert!(Configuration::parse(&arguments("a".repeat(64), "k".repeat(256))).is_ok());
    assert!(Configuration::parse(&arguments("a".repeat(65), "k".repeat(256))).is_err());
    assert!(Configuration::parse(&arguments("a".repeat(64), "k".repeat(257))).is_err());
    for name in [
        "",
        "_",
        "1name",
        "hyphen-name",
        "a b",
        "a\nb",
        "café",
        "-Copt-level",
    ] {
        assert!(
            Configuration::parse(&arguments(name.into(), "valid".into())).is_err(),
            "{name:?}"
        );
    }
    for key in [
        "",
        "a b",
        "a\nb",
        "café",
        "key=value",
        "quoted\"",
        "back\\slash",
    ] {
        assert!(
            Configuration::parse(&arguments("valid".into(), key.into())).is_err(),
            "{key:?}"
        );
    }
}

#[test]
fn incomplete_duplicate_unknown_and_misplaced_flags_are_rejected() {
    for arguments in [
        vec!["--package", "--crate-name", "one"],
        vec!["--package", "--crate-key", "one"],
        vec!["--package", "--crate-name"],
        vec![
            "--package",
            "--crate-name",
            "one",
            "--crate-name",
            "one",
            "--crate-key",
            "key",
        ],
        vec![
            "--package",
            "--crate-name",
            "one",
            "--crate-key",
            "key",
            "--crate-key",
            "key",
        ],
        vec!["--crate-name", "one", "--crate-key", "key"],
        vec!["--input", "file.rs", "--package"],
        vec!["--package", "--package", "value"],
        vec!["--package", "--extern", "dependency=ambient.rlib"],
        vec!["--package", "--sysroot", "other"],
        vec!["--package", "--crate-type", "bin"],
        vec!["--package", "-Aunsafe-code", "ignored"],
    ] {
        assert!(parse(&arguments).is_err(), "{arguments:?}");
    }
}
