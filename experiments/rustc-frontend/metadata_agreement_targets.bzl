"""Isolated pinned compiler metadata evidence; no target generation bypass."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter")

def metadata_agreement_targets(name):
    """Declare the compiler-only agreement probe and regression test.

    Args:
        name: Compiler metadata integration test target.
    """
    sources = [
        "src/compiler_dependencies.rs",
        "src/inputs.rs",
        "test/metadata_probe/main.rs",
        "test/metadata_probe/mapped.rs",
        "test/metadata_probe/preload.rs",
        "test/metadata_probe/snapshot.rs",
    ]
    compiler_adapter(
        name = "metadata_probe",
        srcs = sources,
        crate_root = "test/metadata_probe/main.rs",
        deps = [":compiler_configuration"],
    )
    adapter_format_test(
        name = "metadata_probe_format_test",
        srcs = sources,
    )
    sh_test(
        name = name,
        srcs = ["test/metadata_agreement_test.sh"],
        args = ["$(rootpath :metadata_probe)", "$(rootpath test/metadata_agreement_test.py)"],
        data = [":metadata_probe", "test/metadata_agreement_test.py"],
    )
    sh_test(
        name = "metadata_preload_test",
        srcs = ["test/metadata_agreement_test.sh"],
        args = ["$(rootpath :metadata_probe)", "$(rootpath test/metadata_preload_test.py)"],
        data = [":metadata_probe", "test/metadata_preload_test.py"],
    )
