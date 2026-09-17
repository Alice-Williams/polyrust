"""Focused compiler/documentation proof targets, sharing the pinned adapter."""

load("@rules_cc//cc:defs.bzl", "cc_binary")
load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def documentation_targets(name, copts):
    """Declare the documentation proof suite.

    Args:
        name: Name of the aggregate test suite.
        copts: Strict C flags shared with the other compiler fixtures.
    """
    rust_binary(
        name = "rust_documentation",
        srcs = [
            "fixtures/documentation.rs",
            "fixtures/reference_documentation.rs",
        ],
        compile_data = ["fixtures/documentation.md"],
        crate_root = "fixtures/reference_documentation.rs",
        edition = "2024",
    )
    native.genrule(
        name = "generate_documentation",
        srcs = [
            "fixtures/documentation.md",
            "fixtures/documentation.rs",
        ],
        outs = ["generated/documentation.c"],
        cmd = "$(location :adapter) $(location fixtures/documentation.rs) $@ --input $(location fixtures/documentation.md)",
        tools = [":adapter"],
    )
    for optimization in ["0", "2"]:
        cc_binary(
            name = "c_documentation_o" + optimization,
            srcs = [
                "fixtures/consumer.c",
                ":generate_documentation",
            ],
            copts = copts + ["-O" + optimization],
        )
    sh_test(
        name = "documentation_proof_test",
        srcs = ["proof_test.sh"],
        args = [
            "$(rootpath :adapter)",
            "$(rootpath :rust_documentation)",
            "$(rootpath :c_documentation_o0)",
            "$(rootpath :c_documentation_o2)",
            "$(rootpath fixtures/documentation.rs)",
            "--input",
            "$(rootpath fixtures/documentation.md)",
        ],
        data = [
            ":adapter",
            ":c_documentation_o0",
            ":c_documentation_o2",
            ":rust_documentation",
        ] + native.glob(["fixtures/*"]),
    )
    compiler_adapter(
        name = "documentation_adapter",
        srcs = [
            "src/inputs.rs",
            "test/documentation_assertions.rs",
            "test/documentation_main.rs",
        ] + ["src/source_admission.rs"] + native.glob(["src/c_lower/**/*.rs", "src/source_capabilities/**/*.rs", "src/source_origin/**/*.rs"]),
        crate_root = "test/documentation_main.rs",
        deps = [
            "//crates/backend-c:portable_backend_c",
            "//crates/binary64:portable_binary64",
            "//crates/codegen:portable_codegen",
        ],
    )
    sh_test(
        name = "documentation_ast_test",
        srcs = ["test/provenance_test.sh"],
        args = [
            "$(rootpath :documentation_adapter)",
            "$(rootpath fixtures/documentation.rs)",
            "--input",
            "$(rootpath fixtures/documentation.md)",
        ],
        data = [
            "fixtures/documentation.md",
            "fixtures/documentation.rs",
            ":documentation_adapter",
        ],
    )
    sh_test(
        name = "documentation_include_test",
        srcs = ["test/documentation_include_test.sh"],
        args = [
            "$(rootpath :adapter)",
            "$(rootpath fixtures/documentation.rs)",
        ],
        data = [
            "fixtures/documentation.rs",
            ":adapter",
        ],
    )
    compiler_adapter(
        name = "documentation_sharing_adapter",
        srcs = [
            "src/inputs.rs",
            "test/documentation_sharing_assertions.rs",
            "test/documentation_sharing_main.rs",
        ] + ["src/source_admission.rs"] + native.glob(["src/c_lower/**/*.rs", "src/source_capabilities/**/*.rs", "src/source_origin/**/*.rs"]),
        crate_root = "test/documentation_sharing_main.rs",
        deps = [
            "//crates/backend-c:portable_backend_c",
            "//crates/binary64:portable_binary64",
            "//crates/codegen:portable_codegen",
        ],
    )
    sh_test(
        name = "documentation_sharing_test",
        srcs = ["test/documentation_sharing_test.sh"],
        args = ["$(rootpath :documentation_sharing_adapter)"],
        data = [":documentation_sharing_adapter"],
    )
    sh_test(
        name = "documentation_source_inputs_test",
        srcs = ["test/documentation_source_inputs_test.sh"],
        args = [
            "$(rootpath :adapter)",
            "$(rootpath fixtures/consumer.c)",
        ],
        data = [
            "fixtures/consumer.c",
            ":adapter",
        ],
    )
    native.test_suite(
        name = name,
        tests = [
            ":documentation_ast_test",
            ":documentation_include_test",
            ":documentation_native_matrix_test",
            ":documentation_proof_test",
            ":documentation_sharing_test",
            ":documentation_source_inputs_test",
        ],
    )
