"""Real source/loaded-artifact authentication, including unused dependencies."""
import os
from pathlib import Path
import subprocess
import sys

emitter = Path(sys.argv[1]).resolve()
checker = Path(sys.argv[2]).resolve()
callback_contract = Path(sys.argv[3]).resolve()
for artifact, count in zip(sys.argv[4:], [3, 2], strict=True):
    assert Path(artifact).read_text() == f"checked {count} source crates; no target output published\n"
work = Path(os.environ["TEST_TMPDIR"]) / "source-check"
work.mkdir()
scratch = work / "scratch"
scratch.mkdir()


def source(name, text="pub fn identity(v: i32) -> i32 { v }\n"):
    path = work / (name + ".rs")
    path.write_text(text)
    return path


def crate(name, path, dependencies=(), key=None, logical=None, inputs=()):
    return dict(name=name, key=key or name + ".key", path=path,
                metadata=work / ("lib" + name + ".rmeta"),
                dependencies=dependencies, logical=logical or path.name, inputs=inputs)


def records(root, crates):
    args = ["--root", root["key"]]
    for item in crates:
        args.extend(["--crate", item["name"], item["key"], item["path"],
                     item["logical"], item["metadata"]])
        for path, logical in item["inputs"]:
            args.extend(["--input", path, logical])
        for alias, key in item["dependencies"]:
            args.extend(["--dependency", alias, key])
    return args


def run(root, crates, checking=True):
    return subprocess.run([str(arg) for arg in [
        checker if checking else emitter, *records(root, crates)]],
        capture_output=True, text=True, timeout=60,
        env=dict(os.environ, TMPDIR=str(scratch)))


def accept(root, crates):
    before = {p: p.read_bytes() for p in work.glob("*.rmeta")}
    result = run(root, crates)
    assert result.returncode == 0, result.stderr
    assert result.stdout == f"checked {len(crates)} source crates; no target output published\n"
    assert {p: p.read_bytes() for p in work.glob("*.rmeta")} == before
    assert not list(scratch.iterdir()), "source checker leaked a private scratch stage"
    contract = subprocess.run([str(arg) for arg in [callback_contract, *records(root, crates)]],
        capture_output=True, text=True, timeout=60, env=dict(os.environ, TMPDIR=str(scratch)))
    assert contract.returncode == 0, contract.stderr
    assert contract.stdout == result.stdout
    assert not list(scratch.iterdir()), "callback contract leaked a private scratch stage"


def reject(root, crates, diagnostic=None):
    before = {p: p.read_bytes() for p in work.glob("*.rmeta")}
    result = run(root, crates)
    assert result.returncode != 0, result.stdout
    assert not result.stdout, "failed graph reported a successful prefix"
    if diagnostic:
        assert diagnostic in result.stderr, result.stderr
    assert {p: p.read_bytes() for p in work.glob("*.rmeta")} == before
    assert not list(scratch.iterdir()), "failed source checker leaked a private scratch stage"


def emit(root, crates):
    result = run(root, crates, checking=False)
    assert result.returncode == 0, result.stderr
    assert root["metadata"].stat().st_size > 0


leaf = crate("leaf", source("leaf"))
middle = crate("middle", source("middle"), [("dep", leaf["key"])])
root = crate("root", source("root"), [("dep", middle["key"])])
emit(leaf, [leaf])
emit(middle, [middle, leaf])
# Source checks do not require a root metadata file, and never create one.
accept(leaf, [leaf])
accept(root, [root, middle, leaf])
assert not root["metadata"].exists()
# The same operation accepts an existing root metadata input without replacing it.
emit(root, [root, middle, leaf])
accept(root, [leaf, root, middle])

# Same exported names/signatures are not source-content agreement.
original = leaf["path"].read_text()
for text in [
    "pub fn identity(v: i32) -> i32 { v.wrapping_add(1) }\n",
    "/// Changed documentation.\npub fn identity(v: i32) -> i32 { v }\n",
]:
    leaf["path"].write_text(text)
    reject(root, [root, middle, leaf], "source/metadata identity or content mismatch")
leaf["path"].write_text(original)
changed_key = dict(leaf, key="another.key")
changed_middle = dict(middle, dependencies=[("dep", changed_key["key"])])
reject(root, [root, changed_middle, changed_key], "source/metadata identity or content mismatch")
reject(root, [root, middle, dict(leaf, logical="other.rs")],
       "source/metadata identity or content mismatch")

# Physical relocation with identical logical source mapping remains equivalent.
relocated = source("relocated", original)
accept(root, [root, middle, dict(leaf, path=relocated)])

# Exact file-to-key joins defeat a coordinated swap even with both owners loaded.
other = crate("other", source("other"))
emit(other, [other])
pair = crate("pair", source("pair"),
             [("left", leaf["key"]), ("right", other["key"])])
accept(pair, [pair, leaf, other])

# Equal crate names and even shared source files still have distinct defining keys.
version = dict(leaf, key="leaf.other.key", metadata=work / "libversion.rmeta")
emit(version, [version])
versions = dict(pair, dependencies=[("left", leaf["key"]), ("right", version["key"])])
accept(versions, [versions, leaf, version])
reject(versions, [versions, dict(leaf, metadata=version["metadata"]),
                  dict(version, metadata=leaf["metadata"])],
       "source/metadata identity or content mismatch")

reject(pair, [pair, dict(leaf, metadata=other["metadata"]),
              dict(other, metadata=leaf["metadata"])],
       "source/metadata identity or content mismatch")

# Repeated aliases and a used alias preserve the same defining owner.
aliases = crate("aliases", source("aliases",
    "pub fn identity(v: i32) -> i32 { left::identity(right::identity(v)) }\n"),
    [("left", leaf["key"]), ("right", leaf["key"])])
accept(aliases, [aliases, leaf])

# A diamond checks each owner once, with exact per-node closure (not siblings).
right = crate("right", source("right"), [("dep", leaf["key"])])
emit(right, [right, leaf])
diamond = crate("diamond", source("diamond"),
                [("left", middle["key"]), ("right", right["key"])])
accept(diamond, [diamond, middle, right, leaf])

# Compiler rejection and expansion reads also fail when only a dependency uses them.
leaf["path"].write_text("pub fn identity(v: i32) -> bool { v }\n")
reject(root, [root, middle, leaf], "compiler rejected source crate")
doc = work / "doc.md"
doc.write_text("Declared documentation.")
leaf["path"].write_text('#[doc = include_str!("doc.md")]\n' + original)
reject(root, [root, middle, leaf], "undeclared compiler file input")
with_doc = dict(leaf, inputs=[(doc, "doc.md")], metadata=work / "libdocumented.rmeta")
emit(with_doc, [with_doc])
doc_root = crate("doc_root", source("doc_root"), [("dep", leaf["key"])])
accept(doc_root, [doc_root, with_doc])
doc.write_text("Changed documentation.")
reject(doc_root, [doc_root, with_doc], "source/metadata identity or content mismatch")
leaf["path"].write_text(original)

# Validate the whole filesystem inventory, including unused metadata, before checking.
bad = work / "libbad.rmeta"
bad.write_bytes(b"not metadata")
reject(root, [root, middle, dict(leaf, metadata=bad)])
reject(root, [root, middle, dict(leaf, metadata=work / "libmissing.rmeta")])
hardlink = work / "libsource-alias.rmeta"
os.link(leaf["path"], hardlink)
reject(root, [root, middle, dict(leaf, metadata=hardlink)], "graph metadata aliases a source")
reject(pair, [pair, leaf, dict(other, metadata=hardlink)], "graph metadata aliases a source")
assert not list(work.glob(".polyrust-metadata-*")), "owned stage leaked"
print("source checking: unused chain, used/repeated aliases, diamond, exact artifacts, source/docs/config mutations and no partial publication passed")
