"""Exact compiler-checked binary64 values across three source crates."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def binary64_targets(name):
    """Declare source libraries and independent exact native tests.

    Args:
        name: Clippy gate for all handwritten Rust fixture code.
    """
    owners = [("binary64_leaf", []), ("binary64_middle", ["binary64_leaf"]), ("binary64_root", ["binary64_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "binary64_root" else owner,
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
        name = "rust_binary64_values",
        srcs = ["fixtures/reference_binary64.rs"],
        crate_root = "fixtures/reference_binary64.rs",
        edition = "2024",
        deps = [":binary64_leaf_model", ":binary64_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_binary64_values"])
    rust_c_bundle(name = "generated_c_binary64", crate = ":binary64_root_metadata")
    rust_java_bundle(name = "generated_java_binary64", crate = ":binary64_root_metadata")
    artifacts = [":generated_java_binary64", ":generated_c_binary64", ":rust_binary64_values", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "binary64_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/binary64_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/binary64_native.py",
            "test/binary64_examples.py",
            "test/constant_export_scratch.py",
            "fixtures/binary64_leaf.rs",
            "fixtures/binary64_middle.rs",
            "fixtures/binary64_root.rs",
            "fixtures/reference_binary64.rs",
            "test/binary64_inventory.py",
            "test/binary64_oracle.py",
            "test/binary64_consumers.py",
            "test/binary64_mutations.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "binary64_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/binary64_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/binary64_rejections.py", ":adapter", ":java_adapter", "@bazel_tools//tools/jdk:current_java_runtime"],
    )
