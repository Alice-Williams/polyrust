"""Exact wide scalar values across real Rust crate and target package boundaries."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def bitwise_targets(name):
    """Create source fixtures, real bundles and independent boundary proofs.

    Args:
        name: Fixture Clippy target.
    """
    for owner, dependencies in [("bitwise_leaf", []), ("bitwise_values", ["bitwise_leaf"])]:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner if owner == "bitwise_leaf" else "bitwise_values_model",
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
        name = "rust_bitwise_values",
        srcs = ["fixtures/reference_bitwise.rs"],
        crate_root = "fixtures/reference_bitwise.rs",
        edition = "2024",
        deps = [":bitwise_values_model"],
    )
    rust_clippy_test(name = name, targets = [":bitwise_leaf_model", ":bitwise_values_model", ":rust_bitwise_values"])
    rust_c_bundle(name = "generated_c_bitwise", crate = ":bitwise_values_metadata")
    rust_java_bundle(name = "generated_java_bitwise", crate = ":bitwise_values_metadata")
    artifacts = [":generated_java_bitwise", ":generated_c_bitwise", ":rust_bitwise_values", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "bitwise_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/bitwise_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/bitwise_native.py",
            "test/bitwise_inventory.py",
            "test/i64_inventory.py",
            "test/i64_oracle.py",
            "test/bitwise_oracle.py",
            "test/bitwise_mutations.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "bitwise_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/bitwise_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/bitwise_rejections.py", ":adapter", ":java_adapter"],
    )
