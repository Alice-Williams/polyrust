"""Separate-process identity/relocation proof and native multi-package consumers."""
import json
import os
from pathlib import Path
import subprocess
import sys

adapter, fixture, artifact_a, artifact_b, zig = [Path(value).resolve() for value in sys.argv[1:]]
work = Path(os.environ["TEST_TMPDIR"]) / "crate-identity"
work.mkdir()


def run(*arguments, **kwargs):
    return subprocess.run([str(value) for value in arguments], check=True, capture_output=True, **kwargs)


def snapshot(directory, declared_artifact=False):
    # Bazel runfiles expose the declared tree's regular files through symlinks.
    # Direct CLI destinations must themselves contain newly created regular files.
    assert all(path.is_file() and (declared_artifact or not path.is_symlink()) for path in directory.iterdir())
    return {path.name: path.read_bytes() for path in directory.iterdir()}


def generate(source, destination, name, key):
    run(adapter, source, destination, "--package", "--crate-name", name, "--crate-key", key)
    files = snapshot(destination)
    manifest = json.loads(files["api.json"])
    assert set(files) == {"api.json", manifest["header"], manifest["implementation"]}
    assert len(manifest["functions"]) == 2
    for function in manifest["functions"]:
        crate, declaration = function["id"].split(":")
        assert crate == manifest["root"].split(":")[0]
        assert function["symbol"] == f"poly_fn_{crate}_{declaration}"
    return manifest


packages = []
for label, name, key, artifact in [
    ("a", "identity_fixture", "polyrust.identity.a", artifact_a),
    ("b", "identity_fixture", "polyrust.identity.b", artifact_b),
    ("renamed", "other_identity", "polyrust.identity.a", None),
]:
    directory = work / label
    manifest = generate(fixture, directory, name, key)
    repeated = work / (label + "-repeat")
    assert generate(fixture, repeated, name, key) == manifest
    assert snapshot(directory) == snapshot(repeated)
    if artifact:
        assert snapshot(directory) == snapshot(artifact, declared_artifact=True)
    relocated = work / (label + " relocated")
    relocated.mkdir()
    source = relocated / "different_filename.rs"
    source.write_bytes(fixture.read_bytes())
    generate(source, relocated / "package", name, key)
    assert snapshot(directory) == snapshot(relocated / "package")
    public = [function for function in manifest["functions"] if function["linkage"] == "external"]
    assert len(public) == 1
    root = next(module for module in manifest["modules"] if module["id"] == manifest["root"])
    assert len(root["bindings"]) == 1
    assert root["bindings"][0]["name"] == "identity"
    assert root["bindings"][0]["target"] == public[0]["id"]
    packages.append((directory, manifest, public[0]["symbol"]))

assert len({manifest["root"] for _, manifest, _ in packages}) == 3
assert len({symbol for _, _, symbol in packages}) == 3
assert len({function["symbol"] for _, manifest, _ in packages for function in manifest["functions"]}) == 6

# All identical public operations coexist without linker renaming or body merging.
headers = [f'#include "{manifest["header"]}"\n' for _, manifest, _ in packages]
body = '''#include <inttypes.h>
#include <stdio.h>
int main(void) {
    int32_t value = 0;
    while (scanf("%" SCNd32, &value) == 1) {
'''
for _, _, symbol in packages:
    body += f"        if ({symbol}(value) != value) {{ return 1; }}\n"
body += '''        printf("%" PRId32 "\\n", value);
    }
    return 0;
}
'''
consumers = []
for order, preamble in [("forward", headers), ("reverse", list(reversed(headers)))]:
    path = work / f"consumer-{order}.c"
    path.write_text("".join(preamble + preamble) + body)
    consumers.append(path)
values = [-2147483648, -2147483647, -1000000, -65536, -42, -1, 0, 1, 42, 2147483646, 2147483647]
values += list(range(-4096, 4097))
vectors = ("\n".join(map(str, values)) + "\n").encode()
assert run("gcc-14", "-dumpfullversion").stdout.strip() == b"14.2.0"
common = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes", "-Wmissing-prototypes",
          "-fno-fast-math", "-ffp-contract=off", "-fsigned-char", "-fno-short-enums"]
includes = [argument for directory, _, _ in packages for argument in ("-I", str(directory))]
for optimization in ("0", "2"):
    for compiler in ("gcc", "zig", "asan", "ubsan"):
        command = [zig] if compiler == "zig" else ["gcc-14"]
        flags = []
        if compiler in ("asan", "ubsan"):
            flags = ["-fsanitize=" + ("address" if compiler == "asan" else "undefined"),
                     "-fno-sanitize-recover=all", "-fno-pie", "-no-pie"]
        prefix = work / f"{compiler}-o{optimization}"
        objects = []
        for index, (directory, manifest, symbol) in enumerate(packages):
            obj = prefix.with_suffix(f".{index}.o")
            run(*command, *common, *flags, f"-O{optimization}", *includes, "-c", directory / manifest["implementation"], "-o", obj)
            actual = {line.split()[-1] for line in run("nm", "--defined-only", obj, text=True).stdout.splitlines()
                      if len(line.split()) == 3 and line.split()[1] == "T"}
            assert actual == {symbol}
            objects.append(obj)
        for consumer in consumers:
            obj = prefix.with_suffix(f".{consumer.stem}.o")
            executable = prefix.with_suffix(f".{consumer.stem}")
            run(*command, *common, *flags, f"-O{optimization}", *includes, "-c", consumer, "-o", obj)
            run(*command, *flags, *objects, obj, "-o", executable)
            environment = dict(os.environ, ASAN_OPTIONS="detect_leaks=1:halt_on_error=1", UBSAN_OPTIONS="halt_on_error=1")
            result = run("bash", "-c", 'ulimit -s 1024; exec "$1"', "identity-native", executable,
                         input=vectors, env=environment)
            assert result.stdout == vectors

valid = ["--package", "--crate-name", "identity_fixture", "--crate-key", "polyrust.identity.a"]
negative = [
    ["--package", "--crate-name", "one"], ["--package", "--crate-key", "one"],
    ["--package", "--crate-name"], valid + ["--crate-name", "duplicate"],
    valid + ["--crate-key", "duplicate"], valid[1:], ["--input", str(fixture), "--package"],
    ["--package", "--extern", "dependency=ambient.rlib"], ["--package", "--sysroot", "other"],
    ["--package", "--crate-type", "bin"], ["--package", "-Aunsafe-code", "ignored"],
]
for name in ["", "_", "1bad", "bad-name", "a b", "a\nb", "café", "a" * 65]:
    negative.append(["--package", "--crate-name", name, "--crate-key", "valid"])
for key in ["", "a b", "a\nb", "café", "key=value", "k" * 257]:
    negative.append(["--package", "--crate-name", "valid", "--crate-key", key])
for index, flags in enumerate(negative):
    for existing in (False, True):
        destination = work / f"invalid-{index}-{existing}"
        if existing:
            destination.mkdir()
            (destination / "sentinel").write_bytes(b"unchanged\x00identity")
        result = subprocess.run([str(adapter), str(fixture), str(destination), *flags], capture_output=True)
        assert result.returncode != 0 and b"compiler configuration:" in result.stderr, (flags, result.stderr)
        if existing:
            assert snapshot(destination) == {"sentinel": b"unchanged\x00identity"}
        else:
            assert not destination.exists()
        assert not list(work.glob(".polyrust-stage-*"))
for label, source, diagnostic in [
    ("unstable", "#![feature(rustc_private)]\npub fn identity(value: i32) -> i32 { value }", b"E0554"),
    ("unsafe", "pub unsafe fn identity(value: i32) -> i32 { value }", b"unsafe"),
]:
    path = work / f"{label}.rs"
    path.write_text(source)
    for existing in (False, True):
        destination = work / f"{label}-{existing}"
        if existing:
            destination.mkdir()
            (destination / "sentinel").write_bytes(b"compiler boundary intact")
        result = subprocess.run([str(adapter), str(path), str(destination), "--package",
                                 "--crate-name", "polyrust_rustc_frontend", "--crate-key", "explicit.boundary"],
                                env=dict(os.environ, RUSTC_BOOTSTRAP="polyrust_rustc_frontend"), capture_output=True)
        assert result.returncode != 0 and diagnostic in result.stderr, result.stderr
        if existing:
            assert snapshot(destination) == {"sentinel": b"compiler boundary intact"}
        else:
            assert not destination.exists()
        assert not list(work.glob(".polyrust-stage-*"))
print(f"Three disjoint crate identities, relocation/repetition/Bazel equality, 16 native consumers x {len(values)} inputs, {len(negative)} negative configurations and explicit compiler-boundary rejections passed")
