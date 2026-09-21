"""Ordinary Rust negative-zero composition and independent native proof."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library", "rustfmt_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":bundle_output.bzl", "rust_c_bundle")
load(":java_bundle_output.bzl", "rust_java_bundle")
load(":metadata_output.bzl", "rust_source_metadata")

def negative_zero_targets(name):
    """Declare the source model, generated packages and conformance gate.

    Args:
        name: Clippy gate for the original model and reference.
    """
    source = "//examples/real-world/stdlib-is-negative-zero:rust-source/lib.rs"
    reference = "fixtures/reference_negative_zero.rs"
    native.genrule(
        name = "negative_zero_traced_source",
        testonly = True,
        srcs = [source, "test/negative_zero_rust_trace.py"],
        outs = ["negative_zero_traced.rs"],
        cmd = "python3 $(location test/negative_zero_rust_trace.py) $(location " + source + ") $@",
    )
    for suffix, model in [("", source), ("_traced", ":negative_zero_traced_source")]:
        rust_library(
            name = "negative_zero_model" + suffix,
            testonly = True,
            srcs = [model],
            crate_name = "negative_zero_source",
            crate_root = model,
            edition = "2024",
        )
        rust_binary(
            name = "rust_negative_zero" + suffix,
            testonly = True,
            srcs = [reference],
            crate_root = reference,
            edition = "2024",
            deps = [":negative_zero_model" + suffix],
        )
    rust_clippy_test(name = name, targets = [":negative_zero_model", ":rust_negative_zero"])
    rustfmt_test(
        name = "negative_zero_source_rustfmt_test",
        targets = [":negative_zero_model", ":rust_negative_zero"],
    )
    rust_source_metadata(
        name = "negative_zero_metadata",
        source = source,
        crate_name = "negative_zero_source",
        crate_key = "proof.stdlib.negative_zero.v1",
    )
    rust_c_bundle(name = "generated_c_negative_zero", crate = ":negative_zero_metadata")
    rust_java_bundle(name = "generated_java_negative_zero", crate = ":negative_zero_metadata")
    artifacts = [
        ":generated_java_negative_zero",
        ":generated_c_negative_zero",
        ":rust_negative_zero",
        ":rust_negative_zero_traced",
        "//tools/c:zig_native_oracle",
        "//third_party/stdlib-is-negative-zero:main.js",
        source,
    ]
    sh_test(
        name = "negative_zero_native_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/negative_zero_native.py)"] + ["$(rootpath " + item + ")" for item in artifacts],
        data = [
            "test/negative_zero_native.py",
            "test/negative_zero_inventory.py",
            "test/negative_zero_mutations.py",
            "test/negative_zero_oracle.py",
            "test/negative_zero_consumers.py",
            "test/negative_zero_upstream.mjs",
            "test/negative_zero_examples.py",
            "test/constant_export_scratch.py",
            "test/java_fixture_native.py",
            "test/remainder_source_privacy.py",
            "test/short_circuit_mutations.py",
            reference,
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
