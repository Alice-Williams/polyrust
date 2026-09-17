"""Checked binary64 absolute value across three original Rust crates."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def absolute_targets(name):
    """Declare source, lint, native equivalence and atomic-rejection targets.

    Args:
        name: Clippy gate for handwritten source fixtures and reference consumer.
    """
    owners = [("absolute_leaf", []), ("absolute_middle", ["absolute_leaf"]), ("absolute_root", ["absolute_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "absolute_root" else owner,
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
        name = "rust_absolute_magnitude",
        srcs = ["fixtures/reference_absolute.rs"],
        crate_root = "fixtures/reference_absolute.rs",
        edition = "2024",
        deps = [":absolute_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_absolute_magnitude"])
    rust_c_bundle(name = "generated_c_absolute", crate = ":absolute_root_metadata")
    rust_java_bundle(name = "generated_java_absolute", crate = ":absolute_root_metadata")
    artifacts = [":generated_java_absolute", ":generated_c_absolute", ":rust_absolute_magnitude", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "absolute_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/absolute_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/absolute_native.py",
            "test/absolute_examples.py",
            "test/absolute_inventory.py",
            "test/absolute_oracle.py",
            "test/floating_consumers.py",
            "test/absolute_mutations.py",
            "test/binary64_oracle.py",
            "test/binary64_inventory.py",
            "test/constant_export_scratch.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "fixtures/absolute_leaf.rs",
            "fixtures/absolute_middle.rs",
            "fixtures/absolute_root.rs",
            "fixtures/reference_absolute.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "absolute_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/absolute_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/absolute_rejections.py", ":adapter", ":java_adapter"],
    )
