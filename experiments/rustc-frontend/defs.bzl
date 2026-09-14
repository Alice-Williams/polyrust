"""Build the isolated adapter against Bazel's pinned compiler internals."""

load("@rules_rust//rust:rust_common.bzl", "CrateInfo", "DepInfo")

def _runfile(path):
    return path[3:] if path.startswith("../") else "_main/" + path

def _adapter_impl(ctx):
    tc = ctx.toolchains["@rules_rust//rust:toolchain_type"]
    if tc.version != "1.98.0" or tc.target_triple.str != "x86_64-unknown-linux-gnu":
        fail("expected Rust 1.98.0 Linux x86_64, got %s %s" % (tc.version, tc.target_triple.str))
    dev = ctx.attr._dev[DefaultInfo].files
    metadata = [f for f in dev.to_list() if f.basename.startswith("librustc_middle-")]
    if len(metadata) != 1:
        fail("expected one rustc_middle metadata file")
    binary = ctx.actions.declare_file(ctx.label.name + ".bin")
    args = ctx.actions.args()
    args.add(ctx.file.crate_root)
    if ctx.attr.rustc_cfg:
        args.add("--cfg", ctx.attr.rustc_cfg)
    for cfg in ctx.attr.rustc_cfgs:
        args.add("--cfg", cfg)
    args.add_all([
        "--crate-name=polyrust_rustc_frontend",
        "--edition=2024",
        "--sysroot=" + tc.sysroot,
        "-L" + metadata[0].dirname,
        "-L" + tc.sysroot + "/lib",
        "-Cprefer-dynamic",
        "-Clinker=/usr/bin/gcc-14",
        "-Dwarnings",
        "-o",
        binary.path,
    ])
    libraries = []
    runtime_data = []
    link_metadata = []
    for dependency in ctx.attr.deps:
        crate = dependency[CrateInfo]
        info = dependency[DepInfo]
        if crate.type not in ["lib", "rlib"] or info.transitive_noncrates.to_list():
            fail("compiler adapter dependencies must be Rust libraries without native dependencies")
        link_metadata.append(info.link_search_path_files)
        link_metadata.append(depset([item.linker_flags for item in info.transitive_build_infos.to_list() if item.linker_flags]))
        args.add("--extern", crate.name + "=" + crate.output.path)
        libraries.append(depset([crate.output], transitive = [info.transitive_crate_outputs]))
        runtime_data.append(info.transitive_data)
    library_files = depset(transitive = libraries)
    link_files = depset(transitive = link_metadata)
    args.add_all(sorted({f.dirname: True for f in library_files.to_list()}), format_each = "-Ldependency=%s")
    ctx.actions.run_shell(
        arguments = [tc.clippy_driver.path, binary.path, ctx.attr.expected_error, str(ctx.attr.expected_errors)] + [f.path for f in link_files.to_list()] + ["--", args],
        command = """
compiler="$1"
output="$2"
expected="$3"
expected_count="$4"
shift 4
while [ "$1" != "--" ]; do
    if [ -s "$1" ]; then
        echo "compiler adapter does not yet support nonempty native link metadata: $1" >&2
        exit 1
    fi
    shift
done
shift
if [ -n "$expected" ]; then
    if "$compiler" "$@" > "$output.diagnostic" 2>&1; then
        echo "compile-fail contract unexpectedly compiled" >&2
        exit 1
    fi
    actual_count=$(grep -Fc "$expected" "$output.diagnostic" || true)
    if [ "$actual_count" -ne "$expected_count" ]; then
        cat "$output.diagnostic" >&2
        exit 1
    fi
    touch "$output"
    exit 0
fi
exec "$compiler" "$@"
""",
        inputs = depset(ctx.files.srcs, transitive = [tc.all_files, dev, library_files, link_files]),
        outputs = [binary],
        env = {
            "LD_LIBRARY_PATH": tc.sysroot + "/lib",
            "PATH": "/usr/bin:/bin",
            "RUSTC_BOOTSTRAP": "polyrust_rustc_frontend",
        },
        mnemonic = "RustCompilerAdapterClippy",
    )
    wrapper = ctx.actions.declare_file(ctx.label.name)
    publication = ""
    publisher_files = []
    if ctx.executable.directory_publisher:
        publisher_files.append(ctx.executable.directory_publisher)
        publication = 'publisher_root="$(cd "$root" && pwd -P)"\nexport POLYRUST_DIRECTORY_PUBLISHER="$publisher_root/%s"\n' % _runfile(ctx.executable.directory_publisher.short_path)
    ctx.actions.write(
        wrapper,
        "#!/bin/sh\nexit 0\n" if ctx.attr.expected_error else """#!/usr/bin/env bash
set -euo pipefail
root="${RUNFILES_DIR:-$0.runfiles}"
sysroot="$root/%s"
export LD_LIBRARY_PATH="$sysroot/lib:$sysroot/lib/rustlib/x86_64-unknown-linux-gnu/lib"
unset RUSTC_BOOTSTRAP
%s
exec "$root/%s" "$sysroot" "$@"
""" % (_runfile(tc.sysroot_short_path), publication, _runfile(binary.short_path)),
        is_executable = True,
    )
    runfiles = ctx.runfiles(files = [binary] + publisher_files, transitive_files = depset(transitive = [tc.all_files] + runtime_data))
    if ctx.executable.directory_publisher:
        runfiles = runfiles.merge(ctx.attr.directory_publisher[DefaultInfo].default_runfiles)
    return [DefaultInfo(
        executable = wrapper,
        runfiles = runfiles,
    )]

def _adapter_attrs():
    return {
        "srcs": attr.label_list(allow_files = [".rs"]),
        "crate_root": attr.label(allow_single_file = [".rs"], mandatory = True),
        "deps": attr.label_list(providers = [CrateInfo, DepInfo]),
        "directory_publisher": attr.label(executable = True, cfg = "exec"),
        "rustc_cfg": attr.string(),
        "rustc_cfgs": attr.string_list(),
        "expected_error": attr.string(),
        "expected_errors": attr.int(default = 1),
        "_dev": attr.label(default = "@rustc_dev//:files"),
    }

compiler_adapter = rule(
    implementation = _adapter_impl,
    executable = True,
    attrs = _adapter_attrs(),
    toolchains = ["@rules_rust//rust:toolchain_type"],
)

compiler_adapter_compile_fail_test = rule(
    implementation = _adapter_impl,
    test = True,
    attrs = _adapter_attrs(),
    toolchains = ["@rules_rust//rust:toolchain_type"],
)

def _format_impl(ctx):
    tc = ctx.toolchains["@rules_rust//rust/rustfmt:toolchain_type"]
    output = ctx.actions.declare_file(ctx.label.name + ".checked")
    ctx.actions.run_shell(
        inputs = depset(ctx.files.srcs, transitive = [tc.all_files]),
        outputs = [output],
        arguments = [tc.rustfmt.path, output.path] + [f.path for f in ctx.files.srcs],
        command = '"$1" --check --edition 2024 "${@:3}" && touch "$2"',
        env = {"LD_LIBRARY_PATH": ":".join(sorted({f.dirname: True for f in tc.rustc_lib.to_list()}))},
        mnemonic = "CompilerFrontendRustfmt",
    )
    script = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(script, "#!/bin/sh\nexit 0\n", is_executable = True)
    return [DefaultInfo(executable = script, runfiles = ctx.runfiles(files = [output]))]

adapter_format_test = rule(
    implementation = _format_impl,
    test = True,
    attrs = {"srcs": attr.label_list(allow_files = [".rs"])},
    toolchains = ["@rules_rust//rust/rustfmt:toolchain_type"],
)
