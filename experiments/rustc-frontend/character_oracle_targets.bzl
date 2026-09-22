"""Independent full-domain character truth before target/source admission."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def character_oracle_targets(name):
    """Declare native scalar-domain/order evidence.

    Args:
        name: Independent/native oracle test.
    """
    source = "fixtures/reference_characters.rs"
    native.filegroup(
        name = "character_oracle_support",
        testonly = True,
        srcs = ["test/character_oracle.py"],
        visibility = ["//crates/backend-c:__pkg__", "//crates/backend-java:__pkg__"],
    )
    references = []
    for suffix, opt, checks in [("checked", "0", "yes"), ("optimized", "2", "no")]:
        target = "character_reference_" + suffix
        rust_binary(
            name = target,
            testonly = True,
            srcs = [source],
            crate_root = source,
            edition = "2024",
            rustc_flags = ["-Copt-level=" + opt, "-Coverflow-checks=" + checks],
        )
        references.append(":" + target)
    rust_clippy_test(name = "character_oracle_clippy_test", targets = references)
    rustfmt_test(name = "character_oracle_rustfmt_test", targets = references)
    sh_test(
        name = name,
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/character_oracle_test.py)"] + ["$(rootpath " + reference + ")" for reference in references],
        data = [":character_oracle_support", "test/character_oracle_test.py"] + references,
    )
