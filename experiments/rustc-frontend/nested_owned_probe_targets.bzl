"""Independent pinned compiler observations for nested owned records."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter")

def nested_owned_probe_targets(name):
    """Declare observation targets.

    Args:
        name: Runtime observation target.
    """
    sources = ["src/inputs.rs"] + native.glob(["test/nested_owned_probe/*.rs"])
    compiler_adapter(name = "nested_owned_probe", srcs = sources, crate_root = "test/nested_owned_probe/main.rs")
    adapter_format_test(name = "nested_owned_probe_format_test", srcs = sources + ["fixtures/nested_owned.rs"])
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/nested_owned_probe_test.py)", "$(rootpath :nested_owned_probe)", "$(rootpath fixtures/nested_owned.rs)"], data = ["test/nested_owned_probe_test.py", ":nested_owned_probe", "fixtures/nested_owned.rs"])
