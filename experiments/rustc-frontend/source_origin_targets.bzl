"""Independent compiler metadata proof, with no concrete backend dependency."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def source_origin_targets(name):
    """Declare the source-only provenance test.

    Args:
        name: Test target name.
    """
    compiler_adapter(
        name = "source_origin_probe",
        crate_root = "test/source_origin_main.rs",
        srcs = [
            "src/inputs.rs",
            "test/source_origin_main.rs",
            "test/source_origin_assertions.rs",
            "test/source_constant_facts_support.rs",
            "src/source_capabilities/constant_values.rs",
            "src/source_capabilities/constant_evaluation.rs",
        ] + native.glob(["src/source_origin/**/*.rs"]),
        deps = ["//crates/codegen:portable_codegen", "//crates/binary64:portable_binary64"],
    )
    sh_test(
        name = name,
        srcs = ["test/source_origin_test.sh"],
        args = [
            "$(rootpath :source_origin_probe)",
            "$(rootpath fixtures/documentation.rs)",
            "$(rootpath fixtures/documentation.md)",
        ],
        data = [
            ":source_origin_probe",
            "fixtures/documentation.rs",
            "fixtures/documentation.md",
        ],
    )
