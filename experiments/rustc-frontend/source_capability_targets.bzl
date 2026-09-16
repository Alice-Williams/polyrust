"""Compiler input contracts compile without any target backend dependency."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")

def source_capability_targets(name):
    """Declare an independently cached target-free compiler contract probe.

    Args:
        name: Contract test target name.
    """
    compiler_adapter(
        name = "source_capabilities_probe",
        crate_root = "test/source_capabilities_main.rs",
        srcs = ["test/source_capabilities_main.rs"] + native.glob(["src/source_capabilities/**/*.rs", "src/source_origin/**/*.rs"]),
        deps = ["//crates/codegen:portable_codegen"],
    )
    sh_test(
        name = name,
        srcs = ["test/source_capabilities_test.sh"],
        args = ["$(rootpath :source_capabilities_probe)"],
        data = [":source_capabilities_probe"],
    )
