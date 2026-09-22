"""Original finite-constant source crates and independently generated packages."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def finite_constant_source_targets(name):
    """Declare original Rust libraries, native references and actual C/Java bundles.

    Args:
        name: Native differential test target.
    """
    libraries = []
    for owner, source, dependencies in [
        ("constants", "fixtures/finite_constant_data.rs", []),
        ("second", "fixtures/finite_constant_second.rs", []),
        ("middle", "fixtures/finite_constant_middle.rs", ["constants", "second"]),
        ("root", "fixtures/finite_constant_root.rs", ["middle"]),
    ]:
        library = "finite_constant_" + owner
        libraries.append(":" + library)
        rust_library(
            name = library,
            crate_name = owner,
            crate_root = source,
            srcs = [source],
            edition = "2024",
            deps = [":finite_constant_" + dep for dep in dependencies],
        )
        rust_source_metadata(
            name = library + "_metadata",
            source = source,
            crate_name = owner,
            crate_key = "proof.finite.constant." + owner,
            dependencies = {dep: ":finite_constant_" + dep + "_metadata" for dep in dependencies},
        )
    references = []
    for suffix, opt, checks in [("checked", "0", "yes"), ("optimized", "2", "no")]:
        reference = "finite_constant_source_reference_" + suffix
        references.append(":" + reference)
        rust_binary(
            name = reference,
            testonly = True,
            srcs = ["fixtures/reference_finite_source.rs"],
            edition = "2024",
            rustc_flags = ["-Copt-level=" + opt, "-Coverflow-checks=" + checks],
            deps = [":finite_constant_root"],
        )
    rust_clippy_test(name = "finite_constant_source_clippy_test", targets = libraries + references)
    rustfmt_test(name = "finite_constant_source_rustfmt_test", targets = libraries + references)
    sh_test(
        name = "finite_constant_source_rejection_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/finite_constant_source_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = ["test/finite_constant_source_rejections.py", ":adapter", ":java_adapter"],
    )
    for owner in ["middle", "root"]:
        rust_c_bundle(name = "generated_c_finite_constant_" + owner, crate = ":finite_constant_" + owner + "_metadata")
        rust_java_bundle(name = "generated_java_finite_constant_" + owner, crate = ":finite_constant_" + owner + "_metadata")
    artifacts = [
        ":generated_" + language + "_finite_constant_" + owner
        for owner in ["middle", "root"]
        for language in ["c", "java"]
    ] + references + ["//tools/c:zig_native_oracle"]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/finite_constant_source_native.py)"] + ["$(rootpath " + target + ")" for target in artifacts],
        data = artifacts + [
            "test/finite_constant_source_native.py",
            "test/finite_constant_source_consumers.py",
            "test/finite_constant_source_inventory.py",
            "test/finite_constant_source_truth.py",
            "test/constant_export_native.py",
            "test/constant_export_scratch.py",
            ":finite_constant_oracle_support",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ],
    )
