"""Real Rust-source alias-only and mixed crates, separately compiled in C/Java."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def constant_export_targets(name):
    """Declare the native constant re-export proof.

    Args:
      name: Name of the native cross-language proof test.
    """
    libraries = []
    for owner, source, dependencies in [
        ("constants", "fixtures/public_constant_data.rs", []),
        ("second", "fixtures/constant_export_second.rs", []),
        ("middle", "fixtures/constant_export_middle.rs", ["constants", "second"]),
        ("root", "fixtures/constant_export_root.rs", ["middle"]),
    ]:
        library = "constant_export_" + owner
        libraries.append(":" + library)
        rust_library(
            name = library,
            crate_name = owner,
            crate_root = source,
            srcs = [source],
            edition = "2024",
            deps = [":constant_export_" + dep for dep in dependencies],
        )
        rust_source_metadata(
            name = library + "_metadata",
            source = source,
            crate_name = owner,
            crate_key = "proof.constant.export." + owner,
            dependencies = {dep: ":constant_export_" + dep + "_metadata" for dep in dependencies},
        )
    rust_binary(
        name = "rust_constant_export",
        srcs = ["fixtures/reference_constant_export.rs"],
        edition = "2024",
        deps = [":constant_export_root"],
    )
    rust_clippy_test(name = "constant_export_fixture_clippy_test", targets = libraries + [":rust_constant_export"])
    bundles = []
    for owner in ["middle", "root"]:
        c = "generated_c_constant_export_" + owner
        java = "generated_java_constant_export_" + owner
        rust_c_bundle(name = c, crate = ":constant_export_" + owner + "_metadata")
        rust_java_bundle(name = java, crate = ":constant_export_" + owner + "_metadata")
        bundles.extend([":" + c, ":" + java])
    artifacts = bundles + [":rust_constant_export", "//tools/c:zig_native_oracle"]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/constant_export_native.py)"] + ["$(rootpath " + target + ")" for target in artifacts],
        data = ["test/constant_export_native.py", "@bazel_tools//tools/jdk:current_java_runtime"] + artifacts,
    )
