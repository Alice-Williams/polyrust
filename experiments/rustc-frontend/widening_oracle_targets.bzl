"""Independent signed-widening oracle without target/source admission changes."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def widening_oracle_targets(name):
    """Declare independent native references and exact signed truth.

    Args:
        name: Native/oracle agreement test.
    """
    source = "fixtures/reference_widening.rs"
    support = ["test/widening_oracle.py"]
    native.filegroup(
        name = "widening_oracle_support",
        testonly = True,
        srcs = support,
        visibility = ["//crates/backend-c:__pkg__", "//crates/backend-java:__pkg__"],
    )
    references = []
    for suffix, optimization, checks in [("checked", "0", "yes"), ("optimized", "2", "no")]:
        reference = "widening_reference_" + suffix
        rust_binary(
            name = reference,
            testonly = True,
            srcs = [source],
            crate_root = source,
            edition = "2024",
            rustc_flags = ["-Copt-level=" + optimization, "-Coverflow-checks=" + checks],
        )
        references.append(":" + reference)
    rust_clippy_test(name = "widening_oracle_clippy_test", targets = references)
    rustfmt_test(name = "widening_oracle_rustfmt_test", targets = references)
    sh_test(
        name = name,
        size = "small",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/widening_oracle_test.py)"] + ["$(rootpath " + reference + ")" for reference in references],
        data = support + ["test/widening_oracle_test.py"] + references,
    )
