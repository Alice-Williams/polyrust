"""Real Rust source constant-import diamond, compiled independently in each target."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":defs.bzl", "compiler_adapter")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def constant_import_targets(name):
    """Declare the constant/function diamond and native proof.

    Args:
        name: Native differential test target name.
    """
    libraries = []
    for owner, source, dependencies in [
        ("constants", "fixtures/public_constant_data.rs", []),
        ("left", "fixtures/constant_import_bridge.rs", ["constants"]),
        ("right", "fixtures/constant_import_bridge.rs", ["constants"]),
        ("root", "fixtures/constant_import_root.rs", ["constants", "left", "right"]),
    ]:
        library = "constant_import_" + owner
        libraries.append(":" + library)
        rust_library(
            name = library,
            crate_name = owner,
            crate_root = source,
            srcs = [source],
            edition = "2024",
            deps = [":constant_import_" + dep for dep in dependencies],
        )
        rust_source_metadata(
            name = library + "_metadata",
            source = source,
            crate_name = owner,
            crate_key = "proof.constant.import." + owner,
            dependencies = {dep: ":constant_import_" + dep + "_metadata" for dep in dependencies},
        )
    rust_binary(
        name = "rust_constant_import",
        srcs = ["fixtures/reference_constant_import.rs"],
        edition = "2024",
        deps = [":constant_import_root"],
    )
    rust_clippy_test(
        name = "constant_import_fixture_clippy_test",
        targets = libraries + [":rust_constant_import"],
    )
    rust_c_bundle(name = "generated_c_constant_import", crate = ":constant_import_root_metadata")
    rust_java_bundle(name = "generated_java_constant_import", crate = ":constant_import_root_metadata")
    artifacts = [
        ":generated_c_constant_import",
        ":generated_java_constant_import",
        ":rust_constant_import",
        "//tools/c:zig_native_oracle",
    ]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/constant_import_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/constant_import_native.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )

    compiler_adapter(
        name = "constant_inventory_probe",
        crate_root = "test/constant_inventory_main.rs",
        srcs = ["test/constant_inventory_main.rs", "src/c_lower/functions.rs", "src/java_lower/functions.rs"] + native.glob(["src/source_capabilities/**/*.rs", "src/source_origin/**/*.rs"]),
        deps = ["//crates/backend-c:portable_backend_c", "//crates/codegen:portable_codegen"],
    )
    sh_test(
        name = "constant_import_limits_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/constant_import_limits.py)", "$(rootpath :metadata_emitter)", "$(rootpath :constant_inventory_probe)"],
        data = ["test/constant_import_limits.py", ":metadata_emitter", ":constant_inventory_probe"],
    )
