"""Actual compiler loading through an exact, direct-aliased metadata closure."""
import os
from pathlib import Path
import subprocess
import sys

emitter, leaf, middle, root, leaf_meta, middle_meta, root_meta, alias_meta = [Path(arg).resolve() for arg in sys.argv[1:]]
assert alias_meta.is_file() and alias_meta.stat().st_size > 0
work = Path(os.environ["TEST_TMPDIR"]) / "metadata-closure"
work.mkdir()


def record(name, key, source, metadata, dependencies=(), logical=None):
    args = ["--crate", name, key, source, logical or source.name, metadata]
    for alias, target in dependencies:
        args.extend(["--dependency", alias, target])
    return args


def execute(key, records):
    return subprocess.run([str(arg) for arg in [emitter, "--root", key, *records]],
                          capture_output=True, text=True, timeout=60)


def chain(source, output, dependency=leaf_meta):
    return execute("metadata.root.v1",
        record("metadata_root", "metadata.root.v1", source, output,
               [("renamed_middle", "metadata.middle.v1")], root.name)
        + record("metadata_middle", "metadata.middle.v1", middle, middle_meta,
                 [("renamed_leaf", "metadata.action.v1")])
        + record("metadata_fixture", "metadata.action.v1", leaf, dependency))


direct = work / "libdirect.rmeta"
result = chain(root, direct)
assert result.returncode == 0, result.stderr
assert direct.read_bytes() == root_meta.read_bytes(), "Bazel/direct transitive metadata mismatch"

# Neither temporary metadata staging names nor caller cwd/source location leak.
relocated = work / root.name
relocated.write_bytes(root.read_bytes())
relocated_output = work / "librelocated.rmeta"
result = chain(relocated, relocated_output)
assert result.returncode == 0, result.stderr
assert relocated_output.read_bytes() == root_meta.read_bytes(), "relocated transitive metadata mismatch"

# A dependency available for metadata loading is not a direct source import.
hidden = work / "hidden.rs"
rejected = work / "librejected.rmeta"
for source_text in [
    "pub fn identity(v: i32) -> i32 { metadata_fixture::identity(v) }\n",
    "extern crate metadata_fixture;\npub fn identity(v: i32) -> i32 { metadata_fixture::identity(v) }\n",
]:
    hidden.write_text(source_text)
    result = chain(hidden, rejected)
    assert result.returncode != 0 and "metadata_fixture" in result.stderr, result.stderr
    assert not rejected.exists()

# A substituted leaf cannot satisfy the unchanged middle crate's loaded hash.
changed = work / leaf.name
changed.write_text("pub fn identity(v: i32) -> i32 { v.wrapping_add(1) }\n")
changed_meta = work / "libchanged.rmeta"
result = execute("metadata.action.v1", record("metadata_fixture", "metadata.action.v1", changed, changed_meta))
assert result.returncode == 0, result.stderr
result = chain(root, rejected, changed_meta)
assert result.returncode != 0, "stale transitive metadata accepted"
assert not rejected.exists()

# Missing/malformed/special-file declared inputs do not fall back to ambient files.
for name, content in [("missing", None), ("invalid", b"not metadata"), ("empty", b"")]:
    invalid = work / ("lib" + name + ".rmeta")
    if content is not None:
        invalid.write_bytes(content)
    result = chain(root, rejected, invalid)
    assert result.returncode != 0, result.stderr
    assert not rejected.exists()
fifo = work / "libfifo.rmeta"
os.mkfifo(fifo)
result = chain(root, rejected, fifo)
assert result.returncode != 0 and "regular file" in result.stderr, result.stderr

# Equal compiler names with distinct keys are valid separate direct dependencies.
second_meta = work / "libsecond.rmeta"
result = execute("second.key", record("metadata_fixture", "second.key", changed, second_meta))
assert result.returncode == 0, result.stderr
two = work / "two.rs"
two.write_text("pub fn identity(v: i32) -> i32 { left::identity(right::identity(v)) }\n")
two_output = work / "libtwo.rmeta"
result = execute("two.key", record("two", "two.key", two, two_output,
                                  [("left", "metadata.action.v1"), ("right", "second.key")])
                 + record("metadata_fixture", "metadata.action.v1", leaf, leaf_meta)
                 + record("metadata_fixture", "second.key", changed, second_meta))
assert result.returncode == 0, result.stderr
assert two_output.is_file()
assert not list(work.glob(".polyrust-metadata-*")), "private metadata stages leaked"
print("metadata closure: three declared Bazel actions, exact transitive loading, alias privacy, relocation, stale/invalid rejection and equal-name versions passed")
