"""Compiler-resolved API graph proof, separate from public C package admission."""

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_clippy_test", "rust_library", "rust_test")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def export_graph_targets(name):
    """Declare finite graph and native behavior proofs.

    Args:
        name: Aggregate proof suite name.
    """
    compiler_adapter(
        name = "export_graph_adapter",
        srcs = [
            "src/inputs.rs",
            "test/export_graph_assertions.rs",
            "test/export_graph_main.rs",
            "test/export_graph_mutations.rs",
        ] + ["src/source_admission.rs"] + native.glob(["src/c_lower/**/*.rs", "src/source_capabilities/**/*.rs", "src/source_origin/**/*.rs"]),
        crate_root = "test/export_graph_main.rs",
        deps = [
            "//crates/backend-c:portable_backend_c",
            "//crates/binary64:portable_binary64",
            "//crates/codegen:portable_codegen",
        ],
    )
    sh_test(
        name = "export_graph_ast_test",
        srcs = ["test/provenance_test.sh"],
        args = [
            "$(rootpath :export_graph_adapter)",
            "$(rootpath fixtures/export_graph.rs)",
            "--input",
            "$(rootpath fixtures/export_graph_out.rs)",
        ],
        data = ["fixtures/export_graph.rs", "fixtures/export_graph_out.rs", ":export_graph_adapter"],
    )
    rust_library(
        name = "export_graph_model",
        srcs = ["fixtures/export_graph.rs", "fixtures/export_graph_out.rs"],
        crate_name = "export_graph_model",
        crate_root = "fixtures/export_graph.rs",
        edition = "2024",
    )
    rust_binary(
        name = "rust_export_graph",
        srcs = ["fixtures/reference_export_graph.rs"],
        edition = "2024",
        deps = [":export_graph_model"],
    )
    rust_clippy_test(
        name = "export_graph_clippy_test",
        targets = [":export_graph_model", ":rust_export_graph", ":export_budget_test"],
    )
    native.genrule(
        name = "generate_export_graph",
        srcs = ["fixtures/export_graph.rs", "fixtures/export_graph_out.rs"],
        outs = ["generated/export_graph.c"],
        cmd = "$(location :adapter) $(location fixtures/export_graph.rs) $@ --input $(location fixtures/export_graph_out.rs)",
        tools = [":adapter"],
    )
    sh_test(
        name = "export_graph_native_matrix_test",
        size = "medium",
        srcs = ["test/native_matrix_test.sh"],
        args = [
            "$(rootpath :generate_export_graph)",
            "$(rootpath :rust_export_graph)",
            "$(rootpath fixtures/consumer.c)",
            "$(rootpath fixtures/inputs.txt)",
            "$(rootpath //tools/c:zig_native_oracle)",
        ],
        data = [
            ":generate_export_graph",
            ":rust_export_graph",
            "fixtures/consumer.c",
            "fixtures/inputs.txt",
            "//tools/c:zig_native_oracle",
        ],
    )
    sh_test(
        name = "export_graph_negative_test",
        srcs = ["test/export_graph_negative_test.sh"],
        args = [
            "$(rootpath :adapter)",
            "$(rootpath fixtures/export_graph.rs)",
            "$(rootpath fixtures/export_graph_out.rs)",
        ],
        data = [":adapter", "fixtures/export_graph.rs", "fixtures/export_graph_out.rs"],
    )
    rust_test(
        name = "export_budget_test",
        srcs = ["test/export_budget.rs", "src/source_origin/exports/budget.rs"],
        crate_root = "test/export_budget.rs",
        edition = "2024",
    )
    native.test_suite(
        name = name,
        tests = [
            ":export_budget_test",
            ":export_graph_ast_test",
            ":export_graph_clippy_test",
            ":export_graph_native_matrix_test",
            ":export_graph_negative_test",
        ],
    )
