"""Observe local owned parameter/return representation before admission."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter")

def owned_calls_targets(name):
    """Declare the local-call observation.

    Args:
        name: Observation test target.
    """
    sources = ["src/inputs.rs", "test/owned_calls/main.rs"]
    compiler_adapter(name = "owned_calls_probe", srcs = sources, crate_root = "test/owned_calls/main.rs")
    adapter_format_test(name = "owned_calls_format_test", srcs = sources + ["fixtures/owned_calls.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/owned_calls_test.py)", "$(rootpath :owned_calls_probe)", "$(rootpath fixtures/owned_calls.rs)"], data = ["test/owned_calls_test.py", ":owned_calls_probe", "fixtures/owned_calls.rs"])
