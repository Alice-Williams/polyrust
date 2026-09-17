"""Checked primitive method identities and runtime-free scalar crate bundles."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def wrapping_targets(name):
    """Declare real Rust owners, compiler metadata and independently generated bundles.

    Args:
        name: Fixture lint target.
    """
    models = []
    for owner, dependencies in [("wrapping_leaf", []), ("wrapping_root", ["wrapping_leaf"])]:
        source = "fixtures/" + owner + ".rs"
        models.append(":" + owner + "_model")
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner,
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
        name = "rust_wrapping",
        srcs = ["fixtures/reference_wrapping.rs"],
        crate_root = "fixtures/reference_wrapping.rs",
        edition = "2024",
        deps = [":wrapping_root_model"],
    )
    rust_clippy_test(name = name, targets = models + [":rust_wrapping"])
    rust_c_bundle(name = "generated_c_wrapping", crate = ":wrapping_root_metadata")
    rust_java_bundle(name = "generated_java_wrapping", crate = ":wrapping_root_metadata")

    artifacts = [":generated_java_wrapping", ":generated_c_wrapping", ":rust_wrapping", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "wrapping_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/wrapping_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/wrapping_native.py",
            "test/wrapping_inventory.py",
            "test/wrapping_oracle.py",
            "test/wrapping_consumers.py",
            "test/wrapping_examples.py",
            "test/i64_inventory.py",
            "test/i64_oracle.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "test/constant_export_scratch.py",
            "fixtures/wrapping_leaf.rs",
            "fixtures/wrapping_root.rs",
            "fixtures/reference_wrapping.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )

    sh_test(
        name = "wrapping_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/wrapping_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/wrapping_rejections.py", ":adapter", ":java_adapter"],
    )
