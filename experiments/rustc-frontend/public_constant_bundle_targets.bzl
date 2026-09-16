"""Owned constant bundle metadata and independent native consumers."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":defs.bzl", "compiler_adapter")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def public_constant_bundle_targets(name, adapter_sources):
    """Build constants-only and mixed bundles.

    Args:
        name: Native bundle proof target.
        adapter_sources: Production compiler inputs for collision mutation probe.
    """
    rust_source_metadata(
        name = "public_constant_data_metadata",
        source = "fixtures/public_constant_data.rs",
        crate_name = "constant_data",
        crate_key = "proof.public.constant.data",
    )
    rust_source_metadata(
        name = "public_constant_values_metadata",
        source = "fixtures/public_constant_values.rs",
        crate_name = "constant_values",
        crate_key = "proof.public.constant.values",
        inputs = {"fixtures/public_constant_data.rs": "public_constant_data.rs"},
        dependencies = {"unused_constants": ":public_constant_data_metadata"},
    )
    rust_c_bundle(name = "generated_c_public_constants", crate = ":public_constant_values_metadata")
    rust_java_bundle(name = "generated_java_public_constants", crate = ":public_constant_values_metadata")
    artifacts = [
        ":generated_c_public_constants",
        ":generated_java_public_constants",
        ":rust_public_constant_values",
        "//tools/c:zig_native_oracle",
    ]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/public_constant_bundle_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/public_constant_bundle_native.py",
            "test/public_constant_native.py",
            "test/public_constant_entry.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )

    compiler_adapter(
        name = "c_graph_constant_collision",
        crate_root = "src/main.rs",
        srcs = adapter_sources,
        rustc_cfg = "c_graph_constant_collision",
        directory_publisher = ":directory_publisher",
        deps = [":compiler_configuration", ":directory_publication", "//crates/backend-c:portable_backend_c", "//crates/codegen:portable_codegen"],
    )
    publication = [
        ":adapter",
        ":java_graph_adapter",
        ":c_graph_constant_collision",
        ":c_graph_inventory_contract",
        "fixtures/public_constant_data.rs",
        ":public_constant_data_metadata",
        "fixtures/public_constant_values.rs",
        ":public_constant_values_metadata",
    ]
    sh_test(
        name = "public_constant_bundle_publication_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/public_constant_bundle_publication.py)"] + ["$(rootpath " + item + ")" for item in publication],
        data = ["test/public_constant_bundle_publication.py"] + publication,
    )
