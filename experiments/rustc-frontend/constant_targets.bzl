"""Scalar constant native proofs through real two-crate package generation."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def constant_targets(name):
    """Create fixture libraries, bundles and native/rejection checks.

    Args:
        name: Fixture Clippy target.
    """
    for owner, dependencies in [("constant_leaf", []), ("constant_values", ["constant_leaf"])]:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner if owner == "constant_leaf" else "constant_values_model",
            crate_root = source,
            edition = "2024",
            deps = [":" + dependency + "_model" for dependency in dependencies],
        )
        rust_source_metadata(
            name = owner + "_metadata",
            source = source,
            crate_name = owner,
            crate_key = "proof." + owner + ".v1",
            dependencies = {dependency: ":" + dependency + "_metadata" for dependency in dependencies},
        )
    rust_binary(
        name = "rust_constant_values",
        srcs = ["fixtures/reference_constant.rs"],
        crate_root = "fixtures/reference_constant.rs",
        edition = "2024",
        deps = [":constant_values_model"],
    )
    rust_clippy_test(name = name, targets = [":constant_leaf_model", ":constant_values_model", ":rust_constant_values"])
    rust_c_bundle(name = "generated_c_constant", crate = ":constant_values_metadata")
    rust_java_bundle(name = "generated_java_constant", crate = ":constant_values_metadata")
    artifacts = [":generated_java_constant", ":generated_c_constant", ":rust_constant_values", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "constant_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/constant_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/constant_native.py",
            "test/constant_inventory.py",
            "test/constant_oracle.py",
            "test/i64_inventory.py",
            "test/i64_oracle.py",
            "test/java_fixture_native.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "constant_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/constant_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/constant_rejections.py", ":adapter", ":java_adapter"],
    )
