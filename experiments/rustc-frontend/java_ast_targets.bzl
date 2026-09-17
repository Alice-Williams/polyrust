"""Read-only compiler/AST observations compared with production Java output."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def java_ast_targets(name):
    """Declare the compiler-backed Java AST/public API probe.

    Args:
        name: Test target name.
    """
    compiler_adapter(
        name = "java_ast_probe",
        crate_root = "src/java_main.rs",
        rustc_cfg = "java_ast_probe",
        srcs = ["src/java_main.rs", "src/inputs.rs", "test/java_source_assertions.rs", "test/java_expression_assertions.rs", "test/java_dependency_assertions.rs", "test/package_state_java.rs"] + ["src/source_admission.rs"] + native.glob([
            "src/java_lower/**/*.rs",
            "src/source_capabilities/**/*.rs",
            "src/source_origin/**/*.rs",
        ]),
        deps = [
            ":compiler_configuration",
            "//crates/backend-java:portable_backend_java",
            "//crates/binary64:portable_binary64",
            "//crates/codegen:portable_codegen",
            "//crates/diagnostics:portable_diagnostics",
        ],
    )
    sh_test(
        name = name,
        srcs = ["test/java_source_native.sh"],
        args = [
            "$(rootpath test/java_source_ast.py)",
            "$(rootpath :java_ast_probe)",
            "$(rootpath :java_adapter)",
            "$(rootpath fixtures/model.rs)",
        ],
        data = [
            ":java_ast_probe",
            ":java_adapter",
            "test/java_source_ast.py",
            "test/java_source_native.py",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + native.glob(["fixtures/*"]),
    )
