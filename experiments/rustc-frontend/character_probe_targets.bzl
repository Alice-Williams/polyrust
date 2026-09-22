"""Original source-type authentication probes, without production runtime switches."""

load("@rules_shell//shell:sh_test.bzl", "sh_test")
load(":defs.bzl", "compiler_adapter")
load(":java_graph_targets.bzl", "java_graph_sources")

def character_probe_targets(name, c_sources):
    """Declare atomic metadata-corruption and unsupported-source controls.

    Args:
        name: Original type authentication test.
        c_sources: Exact production C adapter source inventory.
    """
    for language in ["c", "java"]:
        compiler_adapter(
            name = "character_" + language + "_probe",
            crate_root = "src/main.rs" if language == "c" else "src/java_main.rs",
            srcs = (c_sources if language == "c" else java_graph_sources()) + ["test/character_type_probe.rs"],
            rustc_cfgs = ["character_source_probe"] + (["java_graph"] if language == "java" else []),
            directory_publisher = ":directory_publisher",
            deps = [
                ":compiler_configuration",
                ":directory_publication",
                "//crates/backend-" + language + ":portable_backend_" + language,
                "//crates/binary64:portable_binary64",
                "//crates/codegen:portable_codegen",
                "//crates/diagnostics:portable_diagnostics",
            ] + ([":java_bundle"] if language == "java" else []),
        )
    artifacts = [
        ":adapter",
        ":java_graph_adapter",
        ":character_c_probe",
        ":character_java_probe",
        ":generated_c_character_root",
        ":generated_java_character_root",
    ] + ["fixtures/character_" + owner + ".rs" for owner in ["leaf", "middle", "root"]] + [
        ":character_" + owner + "_metadata"
        for owner in ["leaf", "middle", "root"]
    ]
    sh_test(
        name = name,
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/character_source_ast.py)"] + ["$(rootpath " + target + ")" for target in artifacts],
        data = artifacts + ["test/character_source_ast.py"],
    )
    sh_test(
        name = "character_source_rejections_test",
        size = "large",
        srcs = ["test/java_source_native.sh"],
        args = ["$(rootpath test/character_source_rejections.py)", "$(rootpath :adapter)", "$(rootpath :java_adapter)"],
        data = [":adapter", ":java_adapter", "test/character_source_rejections.py"],
    )
