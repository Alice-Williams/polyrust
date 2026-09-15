"""Pinned observations for conditional initialization and partial field moves."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter")

def conditional_owned_probe_targets(name):
    """Declare the compiler observation experiment.

    Args:
        name: Observation runtime test.
    """
    sources = ["src/inputs.rs"] + native.glob(["test/conditional_owned_probe/*.rs"])
    compiler_adapter(name = "conditional_owned_probe", srcs = sources, crate_root = "test/conditional_owned_probe/main.rs")
    adapter_format_test(name = "conditional_owned_probe_format_test", srcs = sources + ["fixtures/conditional_owned.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/conditional_owned_probe_test.py)", "$(rootpath :conditional_owned_probe)", "$(rootpath fixtures/conditional_owned.rs)"], data = ["test/conditional_owned_probe_test.py", ":conditional_owned_probe", "fixtures/conditional_owned.rs"])
