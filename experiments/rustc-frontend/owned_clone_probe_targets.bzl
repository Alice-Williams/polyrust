"""Observe concrete standard clone resolution before admitting it."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter")

def owned_clone_probe_targets(name):
    """Declare clone observations.

    Args:
        name: Observation test target.
    """
    sources = ["src/inputs.rs"] + native.glob(["test/owned_clone_probe/*.rs"])
    compiler_adapter(name = "owned_clone_observer", srcs = sources, crate_root = "test/owned_clone_probe/main.rs")
    adapter_format_test(name = "owned_clone_probe_format_test", srcs = sources + ["fixtures/owned_clone.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/owned_clone_probe_test.py)", "$(rootpath :owned_clone_observer)", "$(rootpath fixtures/owned_clone.rs)"], data = ["test/owned_clone_probe_test.py", ":owned_clone_observer", "fixtures/owned_clone.rs"])
