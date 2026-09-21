"""Checked binary64 remainder value across three original Rust crates."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def remainder_source_targets(name):
    """Declare original source, lint and generated bundle targets.

    Args:
        name: Clippy gate for handwritten source fixtures and reference consumer.
    """
    owners = [("remainder_leaf", []), ("remainder_middle", ["remainder_leaf"]), ("remainder_root", ["remainder_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "remainder_root" else owner,
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
        name = "rust_remainder_source",
        srcs = ["fixtures/reference_remainder_source.rs"],
        crate_root = "fixtures/reference_remainder_source.rs",
        edition = "2024",
        deps = [":remainder_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_remainder_source"])
    native.genrule(
        name = "remainder_traced_leaf_source",
        testonly = True,
        srcs = ["fixtures/remainder_leaf.rs", "test/remainder_rust_trace.py"],
        outs = ["remainder_traced_leaf.rs"],
        cmd = "python3 $(location test/remainder_rust_trace.py) $(location fixtures/remainder_leaf.rs) $@",
    )
    for owner, dependencies in owners:
        source = ":remainder_traced_leaf_source" if owner == "remainder_leaf" else "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_traced",
            testonly = True,
            srcs = [source],
            crate_name = owner + "_model" if owner == "remainder_root" else owner,
            crate_root = source,
            edition = "2024",
            deps = [":" + dependency + "_traced" for dependency in dependencies],
        )
    rust_binary(
        name = "rust_remainder_traced",
        testonly = True,
        srcs = ["fixtures/reference_remainder_source.rs"],
        crate_root = "fixtures/reference_remainder_source.rs",
        edition = "2024",
        deps = [":remainder_root_traced"],
    )
    rust_c_bundle(name = "generated_c_remainder", crate = ":remainder_root_metadata")
    rust_java_bundle(name = "generated_java_remainder", crate = ":remainder_root_metadata")
    artifacts = [":generated_java_remainder", ":generated_c_remainder", ":rust_remainder_source", ":rust_remainder_traced", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "remainder_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/remainder_source_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/remainder_source_native.py",
            "test/remainder_source_examples.py",
            "test/remainder_source_inventory.py",
            "test/remainder_source_docs.py",
            "test/remainder_source_privacy.py",
            "test/remainder_source_oracle.py",
            "test/arithmetic_source_consumers.py",
            "test/remainder_source_mutations.py",
            "test/binary64_oracle.py",
            "test/binary64_inventory.py",
            "test/remainder_oracle.py",
            "test/remainder_faults.py",
            "test/arithmetic_oracle.py",
            "test/arithmetic_cases.py",
            "test/arithmetic_source_mutations.py",
            "test/remainder_cases.py",
            "test/constant_export_scratch.py",
            "test/java_fixture_native.py",
            "test/short_circuit_mutations.py",
            "fixtures/remainder_leaf.rs",
            "fixtures/remainder_middle.rs",
            "fixtures/remainder_root.rs",
            "fixtures/reference_remainder_source.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )

    sh_test(
        name = "remainder_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/remainder_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/remainder_rejections.py", ":adapter", ":java_adapter"],
    )
