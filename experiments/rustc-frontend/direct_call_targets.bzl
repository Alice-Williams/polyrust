"""Resolved Rust-source calls reuse the certified C target pipeline."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def direct_call_targets(name):
    """Declare native differential call tests.

    Args:
        name: The aggregate proof suite name.
    """
    rust_library(
        name = "direct_calls_model",
        srcs = ["fixtures/direct_calls.rs", "fixtures/direct_calls_out.rs"],
        crate_name = "direct_calls_model",
        crate_root = "fixtures/direct_calls.rs",
        edition = "2024",
    )
    rust_binary(
        name = "rust_direct_calls",
        srcs = ["fixtures/reference_direct_calls.rs"],
        edition = "2024",
        deps = [":direct_calls_model"],
    )
    rust_clippy_test(
        name = "direct_calls_clippy_test",
        targets = [":direct_calls_model", ":rust_direct_calls"],
    )
    native.genrule(
        name = "generate_direct_calls",
        srcs = ["fixtures/direct_calls.rs", "fixtures/direct_calls_out.rs"],
        outs = ["generated/direct_calls.c"],
        cmd = "$(location :adapter) $(location fixtures/direct_calls.rs) $@ --input $(location fixtures/direct_calls_out.rs)",
        tools = [":adapter"],
    )
    sh_test(
        name = "direct_calls_native_matrix_test",
        size = "medium",
        srcs = ["test/native_matrix_test.sh"],
        args = [
            "$(rootpath :generate_direct_calls)",
            "$(rootpath :rust_direct_calls)",
            "$(rootpath fixtures/consumer.c)",
            "$(rootpath fixtures/inputs.txt)",
            "$(rootpath //tools/c:zig_native_oracle)",
        ],
        data = [
            ":generate_direct_calls",
            ":rust_direct_calls",
            "fixtures/consumer.c",
            "fixtures/inputs.txt",
            "//tools/c:zig_native_oracle",
        ],
    )
    native.test_suite(
        name = name,
        tests = [":direct_calls_ast_test", ":direct_calls_clippy_test", ":direct_calls_native_matrix_test", ":direct_calls_negative_test", ":direct_calls_symbols_test"],
    )
    compiler_adapter(
        name = "direct_calls_adapter",
        srcs = ["src/inputs.rs", "test/direct_call_main.rs", "test/direct_call_assertions.rs"] + ["src/source_admission.rs"] + native.glob(["src/c_lower/**/*.rs", "src/source_capabilities/**/*.rs", "src/source_origin/**/*.rs"]),
        crate_root = "test/direct_call_main.rs",
        deps = ["//crates/backend-c:portable_backend_c", "//crates/codegen:portable_codegen"],
    )
    sh_test(
        name = "direct_calls_ast_test",
        srcs = ["test/provenance_test.sh"],
        args = ["$(rootpath :direct_calls_adapter)", "$(rootpath fixtures/direct_calls.rs)", "--input", "$(rootpath fixtures/direct_calls_out.rs)"],
        data = [":direct_calls_adapter", "fixtures/direct_calls.rs", "fixtures/direct_calls_out.rs"],
    )
    sh_test(
        name = "direct_calls_negative_test",
        srcs = ["test/direct_call_negative_test.sh"],
        args = ["$(rootpath :adapter)"],
        data = [":adapter"],
    )
    sh_test(
        name = "direct_calls_symbols_test",
        srcs = ["test/direct_call_symbols_test.sh"],
        args = ["$(rootpath :generate_direct_calls)", "$(rootpath //tools/c:zig_native_oracle)"],
        data = [":generate_direct_calls", "//tools/c:zig_native_oracle"],
    )
