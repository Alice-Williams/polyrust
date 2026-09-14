"""Public package and same-spelling production bundle differential proofs."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def java_fixture_targets(name):
    """Declare independent fixture proofs and inspectable production bundles.

    Args:
        name: Suite containing the two full-corpus native proofs.
    """
    rust_library(
        name = "same_spelling_model",
        srcs = ["fixtures/java_same_spelling.rs"],
        crate_root = "fixtures/java_same_spelling.rs",
        edition = "2024",
    )
    rust_binary(
        name = "rust_same_spelling",
        srcs = ["fixtures/reference_same_spelling.rs"],
        crate_root = "fixtures/reference_same_spelling.rs",
        edition = "2024",
        deps = [":same_spelling_model"],
    )
    rust_clippy_test(
        name = "same_spelling_clippy_test",
        targets = [":same_spelling_model", ":rust_same_spelling"],
    )
    tests = []
    for fixture, source, reference in [
        ("public_package", "public_package", ":rust_public_package"),
        ("same_spelling", "java_same_spelling", ":rust_same_spelling"),
    ]:
        metadata = "java_fixture_" + fixture + "_metadata"
        rust_source_metadata(
            name = metadata,
            source = "fixtures/" + source + ".rs",
            crate_name = fixture,
            crate_key = "java.fixture." + fixture + ".v1",
        )
        java = "generated_java_" + fixture + "_bundle"
        c = "generated_c_" + fixture + "_bundle"
        rust_java_bundle(name = java, crate = ":" + metadata)
        rust_c_bundle(name = c, crate = ":" + metadata)
        target = "java_fixture_" + fixture + "_test"
        tests.append(":" + target)
        artifacts = [":" + java, ":" + c, reference, "fixtures/inputs.txt", "//tools/c:zig_native_oracle"]
        sh_test(
            name = target,
            size = "medium",
            srcs = ["test/java_source_native.sh"],
            args = ["$(rootpath test/java_fixture_native.py)", fixture] + ["$(rootpath " + artifact + ")" for artifact in artifacts],
            data = ["test/java_fixture_native.py", "@bazel_tools//tools/jdk:current_java_runtime"] + artifacts,
        )
    native.test_suite(name = name, tests = tests)
