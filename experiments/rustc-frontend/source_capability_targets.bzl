"""Compiler input contracts compile without any target backend dependency."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter", "compiler_adapter_compile_fail_test")

def source_capability_targets(name):
    """Declare an independently cached target-free compiler contract probe.

    Args:
        name: Contract test target name.
    """
    sources = [
        "test/source_capabilities_main.rs",
        "test/constant_domains.rs",
    ] + native.glob(["src/source_capabilities/**/*.rs", "src/source_origin/**/*.rs"])
    compiler_adapter(
        name = "source_capabilities_probe",
        crate_root = "test/source_capabilities_main.rs",
        srcs = sources,
        deps = ["//crates/codegen:portable_codegen"],
    )
    sh_test(
        name = name,
        srcs = ["test/source_capabilities_test.sh"],
        args = ["$(rootpath :source_capabilities_probe)"],
        data = [":source_capabilities_probe"],
    )

    for direction in ["literal_to_constant", "constant_to_literal"]:
        compiler_adapter_compile_fail_test(
            name = "constant_domain_" + direction + "_test",
            crate_root = "test/source_capabilities_main.rs",
            srcs = sources,
            rustc_cfg = "constant_domain_" + direction,
            expected_error = "error[E0308]",
            deps = ["//crates/codegen:portable_codegen"],
        )
