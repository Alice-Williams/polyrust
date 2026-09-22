"""Independent constant proof, without target or compiler-source admission."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def finite_constant_oracle_targets(name):
    """Declare native compile-time values and integer-only expectations.

    Args:
        name: Native/oracle agreement test.
    """
    source = "fixtures/reference_finite_constants.rs"
    support = ["test/arithmetic_oracle.py", "test/finite_constant_oracle.py", "test/finite_constant_faults.py"]
    native.filegroup(
        name = "finite_constant_oracle_support",
        testonly = True,
        srcs = support,
        visibility = ["//crates/backend-c:__pkg__", "//crates/backend-java:__pkg__"],
    )
    references = []
    for suffix, optimization, checks in [("checked", "0", "yes"), ("optimized", "2", "no")]:
        reference = "finite_constant_reference_" + suffix
        rust_binary(
            name = reference,
            testonly = True,
            srcs = [source],
            crate_root = source,
            edition = "2024",
            rustc_flags = ["-Copt-level=" + optimization, "-Coverflow-checks=" + checks],
        )
        references.append(":" + reference)
    rust_clippy_test(name = "finite_constant_oracle_clippy_test", targets = references)
    rustfmt_test(name = "finite_constant_oracle_rustfmt_test", targets = references)
    sh_test(
        name = name,
        size = "small",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/finite_constant_oracle_test.py)"] + ["$(rootpath " + reference + ")" for reference in references],
        data = support + ["test/finite_constant_oracle_test.py"] + references,
    )
