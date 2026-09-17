"""Checked binary64 NaN classification across three original Rust crates."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def nan_targets(name):
    """Declare source, lint, native equivalence and atomic-rejection targets.

    Args:
        name: Clippy gate for handwritten source fixtures and reference consumer.
    """
    owners = [("nan_leaf", []), ("nan_middle", ["nan_leaf"]), ("nan_root", ["nan_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "nan_root" else owner,
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
        name = "rust_nan_classification",
        srcs = ["fixtures/reference_nan.rs"],
        crate_root = "fixtures/reference_nan.rs",
        edition = "2024",
        deps = [":nan_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_nan_classification"])
    rust_c_bundle(name = "generated_c_nan", crate = ":nan_root_metadata")
    rust_java_bundle(name = "generated_java_nan", crate = ":nan_root_metadata")
    artifacts = [":generated_java_nan", ":generated_c_nan", ":rust_nan_classification", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "nan_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/nan_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/nan_native.py",
            "test/nan_examples.py",
            "test/nan_inventory.py",
            "test/nan_oracle.py",
            "test/nan_consumers.py",
            "test/nan_mutations.py",
            "test/binary64_oracle.py",
            "test/binary64_inventory.py",
            "test/constant_export_scratch.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "fixtures/nan_leaf.rs",
            "fixtures/nan_middle.rs",
            "fixtures/nan_root.rs",
            "fixtures/reference_nan.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "nan_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/nan_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/nan_rejections.py", ":adapter", ":java_adapter"],
    )
