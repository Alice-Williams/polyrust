"""Feature-preserving runtime migration inventory and real bundle guards."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")

def runtime_parity_targets():
    """Keep inventory drift independent from generated artifact checks."""
    inventory_inputs = [
        "test/runtime_parity_inventory.json",
        "//crates/backend-java:src/capabilities/mod.rs",
        "//crates/backend-java:src/dialect/runtime_helpers.rs",
        "//crates/backend-c:src/generator.rs",
        "//crates/backend-c:src/runtime.h",
        "//crates/backend-c:src/runtime.c",
    ]
    sh_test(
        name = "runtime_parity_inventory_test",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/runtime_parity_inventory.py)"] + ["$(rootpath " + item + ")" for item in inventory_inputs],
        data = ["test/runtime_parity_inventory.py"] + inventory_inputs,
    )
    bundles = [":generated_crate_proof", ":generated_java_crate_proof"]
    sh_test(
        name = "runtime_free_bundles_test",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/runtime_free_bundles.py)"] + ["$(rootpath " + item + ")" for item in bundles],
        data = ["test/runtime_free_bundles.py"] + bundles,
    )
