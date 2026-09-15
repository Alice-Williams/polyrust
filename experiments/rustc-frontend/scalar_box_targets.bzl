"""Scalar-record Box construction and executable registration proof."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "adapter_format_test", "compiler_adapter", "compiler_adapter_compile_fail_test")

def scalar_box_targets(name):
    """Declare the independent boxed-record operation proof.

    Args:
        name: Runtime proof target.
    """
    sources = ["src/inputs.rs", "src/source_capabilities/contracts.rs", "test/owned_record/mapping.rs"] + native.glob([
        "src/owned_source/*.rs",
        "test/scalar_box/*.rs",
    ])
    fixtures = ["fixtures/scalar_box.rs", "fixtures/scalar_box_limits.rs"]
    compiler_adapter(name = "scalar_box_probe", srcs = sources, crate_root = "test/scalar_box/main.rs")
    adapter_format_test(name = "scalar_box_format_test", srcs = sources + fixtures)
    sh_test(name = name, srcs = ["test/java_source_native.sh"], args = ["$(rootpath test/scalar_box_test.py)", "$(rootpath :scalar_box_probe)"] + ["$(rootpath " + f + ")" for f in fixtures], data = ["test/scalar_box_test.py", ":scalar_box_probe"] + fixtures)
    for case, error, count in [
        ("missing", "error[E0277]", 1),
        ("missing_base", "error[E0599]", 1),
        ("duplicate", "error[E0599]", 1),
        ("wrong_capability", "error[E0271]", 1),
        ("wrong_context", "error[E0271]", 2),
        ("wrong_output", "error[E0271]", 2),
        ("wrong_input", "error[E0308]", 1),
        ("private_input", "error[E0451]", 1),
        ("private_payload", "error[E0451]", 1),
        ("private_field", "error[E0451]", 1),
    ]:
        compiler_adapter_compile_fail_test(name = "scalar_box_" + case + "_test", srcs = sources, crate_root = "test/scalar_box/main.rs", rustc_cfg = "scalar_box_" + case, expected_error = error, expected_errors = count)
