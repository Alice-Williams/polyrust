"""Independent modular arithmetic support; no frontend admission changes."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def wrapping_add_oracle_targets(name):
    """Declare native references and the independent integer-oracle gate.

    Args:
        name: Native/oracle agreement test.
    """
    source = "fixtures/reference_wrapping_add.rs"
    references = []
    for suffix, optimization, checks in [("checked", "0", "yes"), ("optimized", "2", "no")]:
        reference = "wrapping_add_reference_" + suffix
        rust_binary(
            name = reference,
            testonly = True,
            srcs = [source],
            crate_root = source,
            edition = "2024",
            rustc_flags = ["-Copt-level=" + optimization, "-Coverflow-checks=" + checks],
        )
        references.append(":" + reference)
    rust_clippy_test(name = "wrapping_add_oracle_clippy_test", targets = references)
    rustfmt_test(name = "wrapping_add_oracle_rustfmt_test", targets = references)
    sh_test(
        name = name,
        size = "small",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/wrapping_add_oracle_test.py)"] + ["$(rootpath " + reference + ")" for reference in references],
        data = ["test/wrapping_add_oracle.py", "test/wrapping_add_oracle_test.py"] + references,
    )
