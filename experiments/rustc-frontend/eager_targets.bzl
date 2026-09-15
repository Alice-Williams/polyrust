"""Eager Boolean native proofs through real two-crate package generation."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def eager_targets(name):
    """Create fixture libraries, bundles and native/rejection checks.

    Args:
        name: Fixture Clippy target.
    """
    for owner, dependencies in [("eager_leaf", []), ("eager_values", ["eager_leaf"])]:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner if owner == "eager_leaf" else "eager_values_model",
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
        name = "rust_eager_values",
        srcs = ["fixtures/reference_eager.rs"],
        crate_root = "fixtures/reference_eager.rs",
        edition = "2024",
        deps = [":eager_values_model"],
    )
    rust_clippy_test(name = name, targets = [":eager_leaf_model", ":eager_values_model", ":rust_eager_values"])
    rust_c_bundle(name = "generated_c_eager", crate = ":eager_values_metadata")
    rust_java_bundle(name = "generated_java_eager", crate = ":eager_values_metadata")
    artifacts = [":generated_java_eager", ":generated_c_eager", ":rust_eager_values", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "eager_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/eager_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/eager_native.py",
            "test/eager_inventory.py",
            "test/eager_oracle.py",
            "test/eager_mutations.py",
            "test/i64_inventory.py",
            "test/i64_oracle.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = "eager_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/eager_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/eager_rejections.py", ":adapter", ":java_adapter"],
    )
