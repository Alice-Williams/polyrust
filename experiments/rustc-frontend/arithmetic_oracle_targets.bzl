"""Independent arithmetic expectations, before either target is admitted."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def arithmetic_oracle_targets():
    """Compare integer/rational expectations with the pinned Rust reference."""
    rust_binary(
        name = "rust_arithmetic_reference",
        srcs = ["fixtures/reference_arithmetic.rs"],
        crate_root = "fixtures/reference_arithmetic.rs",
        edition = "2024",
    )
    rust_clippy_test(
        name = "arithmetic_reference_clippy_test",
        targets = [":rust_arithmetic_reference"],
    )
    sh_test(
        name = "arithmetic_oracle_test",
        srcs = ["test/java_source_native.sh"],
        args = [
            "$(rootpath test/arithmetic_oracle_test.py)",
            "$(rootpath :rust_arithmetic_reference)",
        ],
        data = [
            "test/arithmetic_oracle.py",
            "test/arithmetic_cases.py",
            "test/arithmetic_oracle_test.py",
            ":rust_arithmetic_reference",
        ],
    )
