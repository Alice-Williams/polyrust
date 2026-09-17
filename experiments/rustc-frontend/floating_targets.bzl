"""Checked primitive floating negation across three original Rust crates."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def floating_targets(name):
    """Declare source, lint, native equivalence and atomic-rejection targets.

    Args:
        name: Clippy gate for handwritten source fixtures and reference consumer.
    """
    owners = [("floating_leaf", []), ("floating_middle", ["floating_leaf"]), ("floating_root", ["floating_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "floating_root" else owner,
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
        name = "rust_floating_negation",
        srcs = ["fixtures/reference_floating.rs"],
        crate_root = "fixtures/reference_floating.rs",
        edition = "2024",
        deps = [":floating_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_floating_negation"])
    rust_c_bundle(name = "generated_c_floating", crate = ":floating_root_metadata")
    rust_java_bundle(name = "generated_java_floating", crate = ":floating_root_metadata")
    artifacts = [":generated_java_floating", ":generated_c_floating", ":rust_floating_negation", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "floating_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/floating_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/floating_native.py",
            "test/floating_examples.py",
            "test/floating_inventory.py",
            "test/floating_oracle.py",
            "test/floating_consumers.py",
            "test/floating_mutations.py",
            "test/binary64_oracle.py",
            "test/binary64_inventory.py",
            "test/constant_export_scratch.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "fixtures/floating_leaf.rs",
            "fixtures/floating_middle.rs",
            "fixtures/floating_root.rs",
            "fixtures/reference_floating.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "floating_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/floating_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/floating_rejections.py", ":adapter", ":java_adapter"],
    )
