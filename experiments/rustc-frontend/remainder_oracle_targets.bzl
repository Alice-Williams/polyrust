"""Independent remainder proof, before either target admits the new operation."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def remainder_oracle_targets(name):
    """Declare test-only exact oracle and pinned Rust cross-check.

    Args:
        name: Test-only oracle support filegroup for later target proofs.
    """
    sources = [
        "test/arithmetic_oracle.py",
        "test/arithmetic_cases.py",
        "test/remainder_oracle.py",
        "test/remainder_cases.py",
        "test/remainder_faults.py",
    ]
    rust_binary(
        name = "rust_remainder_reference",
        srcs = ["fixtures/reference_remainder.rs"],
        crate_root = "fixtures/reference_remainder.rs",
        edition = "2024",
    )
    rust_clippy_test(
        name = "remainder_reference_clippy_test",
        targets = [":rust_remainder_reference"],
    )
    sh_test(
        name = "remainder_oracle_test",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/remainder_oracle_test.py)", "$(rootpath :rust_remainder_reference)"],
        data = sources + ["test/remainder_oracle_test.py", ":rust_remainder_reference"],
    )
    native.filegroup(
        name = name,
        testonly = True,
        srcs = sources,
        visibility = ["//crates/backend-c:__pkg__", "//crates/backend-java:__pkg__"],
    )
