"""Original multi-crate Rust and normal runtime-free C/Java packages."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def multiplication_source_targets(name):
    """Declare source, native-reference and generated-package actions.

    Args:
        name: Clippy gate for original fixtures and native reference.
    """
    owners = [("multiplication_leaf", []), ("multiplication_middle", ["multiplication_leaf"]), ("multiplication_root", ["multiplication_middle"])]
    for owner, dependencies in owners:
        source = "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_model",
            srcs = [source],
            crate_name = owner + "_model" if owner == "multiplication_root" else owner,
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
        name = "rust_multiplication_source",
        srcs = ["fixtures/reference_multiplication_source.rs"],
        crate_root = "fixtures/reference_multiplication_source.rs",
        edition = "2024",
        deps = [":multiplication_root_model"],
    )
    rust_clippy_test(name = name, targets = [":" + owner + "_model" for owner, _ in owners] + [":rust_multiplication_source"])
    native.genrule(
        name = "multiplication_traced_leaf_source",
        testonly = True,
        srcs = ["fixtures/multiplication_leaf.rs", "test/addition_rust_trace.py"],
        outs = ["multiplication_traced_leaf.rs"],
        cmd = "python3 $(location test/addition_rust_trace.py) $(location fixtures/multiplication_leaf.rs) $@",
    )
    for owner, dependencies in owners:
        source = ":multiplication_traced_leaf_source" if owner == "multiplication_leaf" else "fixtures/" + owner + ".rs"
        rust_library(
            name = owner + "_traced",
            testonly = True,
            srcs = [source],
            crate_name = owner + "_model" if owner == "multiplication_root" else owner,
            crate_root = source,
            edition = "2024",
            deps = [":" + dependency + "_traced" for dependency in dependencies],
        )
    rust_binary(
        name = "rust_multiplication_traced",
        testonly = True,
        srcs = ["fixtures/reference_multiplication_source.rs"],
        crate_root = "fixtures/reference_multiplication_source.rs",
        edition = "2024",
        deps = [":multiplication_root_traced"],
    )
    rust_c_bundle(name = "generated_c_multiplication", crate = ":multiplication_root_metadata")
    rust_java_bundle(name = "generated_java_multiplication", crate = ":multiplication_root_metadata")
    artifacts = [":generated_java_multiplication", ":generated_c_multiplication", ":rust_multiplication_source", ":rust_multiplication_traced", "//tools/c:zig_native_oracle"]
    sh_test(
        name = "multiplication_rejection_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/multiplication_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/multiplication_rejections.py", ":adapter", ":java_adapter"],
    )
    sh_test(
        name = "multiplication_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/addition_source_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts] + ["multiplication"],
        data = [
            "test/addition_source_native.py",
            "test/addition_source_inventory.py",
            "test/addition_source_consumers.py",
            "test/addition_source_mutations.py",
            "test/addition_source_privacy.py",
            "test/addition_source_examples.py",
            "test/wrapping_mul_oracle.py",
            "test/wrapping_add_oracle.py",
            "test/short_circuit_mutations.py",
            "test/java_fixture_native.py",
            "test/constant_export_scratch.py",
            "fixtures/multiplication_leaf.rs",
            "fixtures/multiplication_middle.rs",
            "fixtures/multiplication_root.rs",
            "fixtures/reference_multiplication_source.rs",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
