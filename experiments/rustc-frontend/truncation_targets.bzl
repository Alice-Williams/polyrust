"""Checked binary64 truncation value across three original Rust crates."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def truncation_targets(name):
    """Declare original source, lint and generated bundle targets.

    Args:
        name: Clippy gate for handwritten source fixtures and reference consumer.
    """
    owners = [("truncation_leaf", []), ("truncation_middle", ["truncation_leaf"]), ("truncation_root", ["truncation_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "truncation_root" else owner,
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
        name = "rust_truncation_integral",
        srcs = ["fixtures/reference_truncation.rs"],
        crate_root = "fixtures/reference_truncation.rs",
        edition = "2024",
        deps = [":truncation_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_truncation_integral"])
    rust_c_bundle(name = "generated_c_truncation", crate = ":truncation_root_metadata")
    rust_java_bundle(name = "generated_java_truncation", crate = ":truncation_root_metadata")
    artifacts = [":generated_java_truncation", ":generated_c_truncation", ":rust_truncation_integral", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "truncation_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/truncation_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/truncation_native.py",
            "test/truncation_examples.py",
            "test/truncation_inventory.py",
            "test/truncation_oracle.py",
            "test/floating_consumers.py",
            "test/truncation_mutations.py",
            "test/binary64_oracle.py",
            "test/truncation_bundle.py",
            "test/constant_export_scratch.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "fixtures/truncation_leaf.rs",
            "fixtures/truncation_middle.rs",
            "fixtures/truncation_root.rs",
            "fixtures/reference_truncation.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "truncation_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/truncation_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/truncation_rejections.py", ":adapter", ":java_adapter"],
    )
