"""Checked binary64 arithmetic value across three original Rust crates."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def arithmetic_source_targets(name):
    """Declare original source, lint and generated bundle targets.

    Args:
        name: Clippy gate for handwritten source fixtures and reference consumer.
    """
    owners = [("arithmetic_leaf", []), ("arithmetic_middle", ["arithmetic_leaf"]), ("arithmetic_root", ["arithmetic_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "arithmetic_root" else owner,
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
        name = "rust_arithmetic_source",
        srcs = ["fixtures/reference_arithmetic_source.rs"],
        crate_root = "fixtures/reference_arithmetic_source.rs",
        edition = "2024",
        deps = [":arithmetic_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_arithmetic_source"])
    rust_c_bundle(name = "generated_c_arithmetic", crate = ":arithmetic_root_metadata")
    rust_java_bundle(name = "generated_java_arithmetic", crate = ":arithmetic_root_metadata")
    artifacts = [":generated_java_arithmetic", ":generated_c_arithmetic", ":rust_arithmetic_source", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "arithmetic_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/arithmetic_source_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/arithmetic_source_native.py",
            "test/arithmetic_source_examples.py",
            "test/arithmetic_source_inventory.py",
            "test/arithmetic_source_oracle.py",
            "test/arithmetic_source_consumers.py",
            "test/arithmetic_source_mutations.py",
            "test/binary64_oracle.py",
            "test/binary64_inventory.py",
            "test/arithmetic_oracle.py",
            "test/arithmetic_cases.py",
            "test/constant_export_scratch.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "fixtures/arithmetic_leaf.rs",
            "fixtures/arithmetic_middle.rs",
            "fixtures/arithmetic_root.rs",
            "fixtures/reference_arithmetic_source.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )

    sh_test(
        name = "arithmetic_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/arithmetic_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/arithmetic_rejections.py", ":adapter", ":java_adapter"],
    )
