"""Scalar constant native proofs through real two-crate package generation."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def local_constant_targets(name):
    """Create fixture libraries, bundles and native/rejection checks.

    Args:
        name: Fixture Clippy target.
    """
    for owner, dependencies in [("local_constant_leaf", []), ("local_constant_values", ["local_constant_leaf"])]:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner if owner == "local_constant_leaf" else "local_constant_values_model",
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
        name = "rust_local_constant_values",
        srcs = ["fixtures/reference_local_constant.rs"],
        crate_root = "fixtures/reference_local_constant.rs",
        edition = "2024",
        deps = [":local_constant_values_model"],
    )
    rust_clippy_test(name = name, targets = [":local_constant_leaf_model", ":local_constant_values_model", ":rust_local_constant_values"])
    rust_c_bundle(name = "generated_c_local_constant", crate = ":local_constant_values_metadata")
    rust_java_bundle(name = "generated_java_local_constant", crate = ":local_constant_values_metadata")
    artifacts = [":generated_java_local_constant", ":generated_c_local_constant", ":rust_local_constant_values", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "local_constant_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/local_constant_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/local_constant_native.py",
            "test/local_constant_inventory.py",
            "test/local_constant_oracle.py",
            "test/i64_inventory.py",
            "test/i64_oracle.py",
            "test/java_fixture_native.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "local_constant_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/local_constant_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/local_constant_rejections.py", ":adapter", ":java_adapter"],
    )
