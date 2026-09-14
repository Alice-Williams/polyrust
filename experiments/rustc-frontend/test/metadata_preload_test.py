"""Prove unused explicit metadata can be loaded before analysis with typed rustc APIs."""
import os
from pathlib import Path
import subprocess
import sys

probe = Path(sys.argv[1]).resolve()
work = Path(os.environ["TEST_TMPDIR"]) / "metadata-preload"
work.mkdir()
leaf = work / "leaf.rs"
leaf.write_text("pub fn identity(v: i32) -> i32 { v }\n")
root = work / "root.rs"
root.write_text("pub fn identity(v: i32) -> i32 { v }\n")
metadata = work / "libleaf.rmeta"


def run(mode, source, output, dependency, name):
    return subprocess.run([str(arg) for arg in [probe, mode, source, output, dependency,
        "--package", "--crate-name", name, "--crate-key", name + ".key"]],
        capture_output=True, text=True, timeout=60)


result = run("emit", leaf, metadata, "-", "leaf")
assert result.returncode == 0, result.stderr
leaf_identity = next(line.split("\t")[3:] for line in result.stdout.splitlines()
                     if line.startswith("crate\tlocal\tleaf\t"))
outputs = []
for mode in ["preload", "preload-emit"]:
    output = work / ("lib" + mode + ".rmeta")
    result = run(mode, root, output, metadata, "root")
    assert result.returncode == 0, result.stderr
    loaded = [line.split("\t")[3:] for line in result.stdout.splitlines()
              if line.startswith("crate\tloaded\tleaf\t")]
    assert loaded == [leaf_identity], result.stdout
    aliases = [line.split("\t")[3:] for line in result.stdout.splitlines()
               if line.startswith("crate\talias\tleaf\t")]
    # The pinned compiler does not populate its source-use alias table when
    # forced loading is the only use. Match the exact loaded metadata artifact.
    assert aliases == [], result.stdout
    paths = [Path(line.split("\t")[2]).resolve() for line in result.stdout.splitlines()
             if line.startswith("metadata\tleaf\t")]
    assert paths == [metadata.resolve()], result.stdout
    outputs.append(next(line for line in result.stdout.splitlines() if line.startswith("crate\tlocal\troot\t")))
assert outputs[0] == outputs[1], "preloaded source analysis/emission hashes differ"
assert not (work / "libpreload.rmeta").exists()
assert (work / "libpreload-emit.rmeta").is_file()

root.write_text("pub fn identity(v: i32) -> i32 { renamed::identity(v) }\n")
result = run("preload", root, "-", metadata, "root")
assert result.returncode == 0, result.stderr
aliases = [line.split("\t")[3:] for line in result.stdout.splitlines()
           if line.startswith("crate\talias\tleaf\t")]
assert aliases == [leaf_identity], result.stdout
root.write_text("pub fn identity(v: i32) -> i32 { v }\n")

bad = work / "libbad.rmeta"
bad.write_bytes(b"invalid unused metadata")
for dependency in [bad, work / "libmissing.rmeta"]:
    output = work / "librejected.rmeta"
    result = run("preload-emit", root, output, dependency, "root")
    assert result.returncode != 0, "unused explicit metadata was not validated"
    assert not output.exists()
print("pre-analysis explicit load: unused dependency identity, local hash agreement and invalid/missing rejection passed")
