"""Backend-independent, pinned compiler drop observation."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter")

def owned_probe_targets(name):
    """Declare the ownership observation boundary and its negative controls.

    Args:
        name: Test target name.
    """
    sources = ["src/inputs.rs"] + native.glob(["test/owned_probe/*.rs"])
    compiler_adapter(
        name = "owned_probe",
        srcs = sources,
        crate_root = "test/owned_probe/main.rs",
    )
    adapter_format_test(
        name = "owned_probe_format_test",
        srcs = sources + ["fixtures/owned_probe.rs"],
    )
    sh_test(
        name = name,
        srcs = ["test/java_source_native.sh"],
        args = [
            "$(rootpath test/owned_probe_test.py)",
            "$(rootpath :owned_probe)",
            "$(rootpath fixtures/owned_probe.rs)",
            "$(rootpath :adapter)",
            "$(rootpath :java_adapter)",
        ],
        data = [
            "test/owned_probe_test.py",
            "fixtures/owned_probe.rs",
            ":owned_probe",
            ":adapter",
            ":java_adapter",
        ],
    )
