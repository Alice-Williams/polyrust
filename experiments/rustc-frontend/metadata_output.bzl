"""Explicit source/configuration provider and independently cached Rust metadata."""

load(":metadata_records.bzl", "crate_graph_arguments")

RustSourceCrateInfo = provider(
    doc = "Declared Rust inputs and metadata, not a compiler/target certificate.",
    fields = {
        "crate_name": "Validated by the closed emitter configuration.",
        "crate_key": "Explicit build disambiguator.",
        "source": "Root source artifact.",
        "source_name": "Stable logical root source name.",
        "inputs": "Declared structs with file and logical_name fields.",
        "metadata": "Declared compiler metadata artifact.",
        "crates": "Closed defining-key map of immutable declared crate records.",
    },
)

def _metadata_impl(ctx):
    output = ctx.actions.declare_file("lib" + ctx.label.name + ".rmeta")
    logical_root = ctx.attr.source_name or ctx.file.source.basename
    declared = []
    for target, logical_name in ctx.attr.inputs.items():
        files = target[DefaultInfo].files.to_list()
        if len(files) != 1:
            fail("each declared source/doc mapping must identify exactly one file")
        declared.append(struct(file = files[0], logical_name = logical_name))
    crates = {}
    dependencies = []
    aliases = {}
    for alias, target in ctx.attr.dependencies.items():
        if alias in aliases:
            fail("duplicate direct Rust dependency alias: " + alias)
        aliases[alias] = True
        info = target[RustSourceCrateInfo]
        dependencies.append(struct(alias = alias, key = info.crate_key))
        for key, record in info.crates.items():
            if key in crates and crates[key] != record:
                fail("conflicting Rust crate descriptions for key: " + key)
            crates[key] = record
            if len(crates) >= 1024:
                fail("Rust metadata graph exceeds 1024 crates including its root")
    if ctx.attr.crate_key in crates:
        fail("root Rust crate key overlaps a dependency")
    crates[ctx.attr.crate_key] = struct(
        crate_name = ctx.attr.crate_name,
        crate_key = ctx.attr.crate_key,
        source = ctx.file.source,
        source_name = logical_root,
        inputs = tuple(declared),
        metadata = output,
        dependencies = tuple(sorted(dependencies, key = _alias)),
    )
    arguments = crate_graph_arguments(ctx, ctx.attr.crate_key, crates)
    metadata_inputs = []
    for key in sorted(crates):
        record = crates[key]
        if key != ctx.attr.crate_key:
            metadata_inputs.append(record.metadata)
    ctx.actions.run(
        executable = ctx.executable._emitter,
        tools = [ctx.attr._emitter[DefaultInfo].files_to_run],
        arguments = [arguments],
        inputs = [ctx.file.source] + [item.file for item in declared] + metadata_inputs,
        outputs = [output],
        env = {"PATH": "/usr/bin:/bin"},
        mnemonic = "CheckedRustMetadata",
    )
    return [
        DefaultInfo(files = depset([output]), runfiles = ctx.runfiles(files = [output])),
        RustSourceCrateInfo(
            crate_name = ctx.attr.crate_name,
            crate_key = ctx.attr.crate_key,
            source = ctx.file.source,
            source_name = logical_root,
            inputs = declared,
            metadata = output,
            crates = crates,
        ),
    ]

def _alias(dependency):
    return dependency.alias

rust_source_metadata = rule(
    implementation = _metadata_impl,
    attrs = {
        "source": attr.label(allow_single_file = [".rs"], mandatory = True),
        "source_name": attr.string(),
        "crate_name": attr.string(mandatory = True),
        "crate_key": attr.string(mandatory = True),
        "inputs": attr.label_keyed_string_dict(allow_files = True),
        "dependencies": attr.string_keyed_label_dict(providers = [RustSourceCrateInfo]),
        "_emitter": attr.label(default = "//experiments/rustc-frontend:metadata_emitter", executable = True, cfg = "exec"),
    },
)
