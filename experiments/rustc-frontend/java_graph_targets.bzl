"""Separately cached compiler-authenticated Java graph adapter and checks."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")
load(":java_bundle_output.bzl", "rust_java_bundle")

def java_graph_targets(name):
    """Declare the non-publishing Java graph adapter and integration test.

    Args:
        name: Compiler graph integration test name.
    """
    sources = java_graph_sources()
    dependencies = [
        ":compiler_configuration",
        ":directory_publication",
        ":java_bundle",
        "//crates/backend-java:portable_backend_java",
        "//crates/binary64:portable_binary64",
        "//crates/codegen:portable_codegen",
        "//crates/diagnostics:portable_diagnostics",
    ]
    mutations = []
    for case in ["", "owner", "declaration", "signature"]:
        target = "java_graph_wrong_" + case if case else "java_graph_adapter"
        compiler_adapter(
            name = target,
            crate_root = "src/java_main.rs",
            rustc_cfg = "java_graph",
            directory_publisher = ":directory_publisher",
            rustc_cfgs = [target] if case else [],
            srcs = sources + (["test/java_graph_mutations.rs"] if case else []),
            deps = dependencies,
        )
        if case:
            mutations.append(":" + target)
    compiler_adapter(
        name = "java_graph_probe",
        crate_root = "src/java_main.rs",
        rustc_cfg = "java_graph",
        rustc_cfgs = ["java_graph_probe"],
        srcs = sources + ["test/java_graph_probe.rs", "test/java_graph_source_probe.rs", "test/java_graph_manifest_probe.rs"],
        deps = dependencies,
    )
    artifacts = [
        value
        for owner in ["leaf", "left", "right", "root"]
        for value in ["fixtures/crate_proof_" + owner + ".rs", ":crate_proof_" + owner + "_metadata"]
    ]
    rust_java_bundle(name = "generated_java_crate_proof", crate = ":crate_proof_root_metadata")
    compiler_adapter(
        name = "java_bundle_transaction_probe",
        crate_root = "src/java_main.rs",
        rustc_cfg = "java_graph",
        srcs = sources,
        deps = dependencies,
    )
    sh_test(
        name = "java_bundle_publication_test",
        size = "medium",
        srcs = ["test/metadata_action_test.sh"],
        args = [
            "$(rootpath test/java_bundle_publication.py)",
            "$(rootpath :java_graph_adapter)",
            "$(rootpath :java_bundle_transaction_probe)",
            "$(rootpath :directory_publisher)",
        ] + ["$(rootpath " + artifact + ")" for artifact in artifacts],
        data = ["test/java_bundle_publication.py", ":java_graph_adapter", ":java_bundle_transaction_probe", ":directory_publisher"] + artifacts,
    )
    sh_test(
        name = "java_graph_native_test",
        size = "medium",
        srcs = ["test/java_source_native.sh"],
        args = [
            "$(rootpath test/java_graph_native.py)",
            "$(rootpath :java_graph_probe)",
            "$(rootpath :java_graph_adapter)",
            "$(rootpath :generated_java_crate_proof)",
            "$(rootpath :rust_crate_proof)",
            "$(rootpath fixtures/inputs.txt)",
        ] + ["$(rootpath " + artifact + ")" for artifact in artifacts],
        data = [
            "test/java_graph_native.py",
            "test/java_manifest_assertions.py",
            ":java_graph_probe",
            ":java_graph_adapter",
            ":generated_java_crate_proof",
            ":rust_crate_proof",
            "fixtures/inputs.txt",
            "@bazel_tools//tools/jdk:current_java_runtime",
        ] + artifacts,
    )
    sh_test(
        name = name,
        size = "medium",
        srcs = ["test/metadata_action_test.sh"],
        args = [
            "$(rootpath test/java_graph_check.py)",
            "$(rootpath :metadata_emitter)",
            "$(rootpath :java_graph_adapter)",
        ] + ["$(rootpath " + target + ")" for target in mutations],
        data = ["test/java_graph_check.py", ":metadata_emitter", ":java_graph_adapter"] + mutations,
    )

def java_graph_sources():
    """Production graph adapter inputs shared with typed compiler probes."""
    return [
        "src/java_main.rs",
        "src/inputs.rs",
        "src/source_admission.rs",
        "src/compiler_dependencies.rs",
        "src/metadata_cli.rs",
        "src/metadata_dependencies.rs",
        "src/metadata_stage.rs",
        "src/source_check.rs",
    ] + native.glob([
        "src/java_graph/**/*.rs",
        "src/java_lower/**/*.rs",
        "src/source_check/**/*.rs",
        "src/source_capabilities/**/*.rs",
        "src/source_origin/**/*.rs",
    ])
