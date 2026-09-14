"""Declared Bazel metadata agrees with source analysis; failures never publish."""
import os
from pathlib import Path
import subprocess
import sys

emitter, probe, fixture, artifact = [Path(value).resolve() for value in sys.argv[1:]]
work = Path(os.environ["TEST_TMPDIR"]) / "metadata-action"
work.mkdir()
KEY = "metadata.action.v1"


def execute(*arguments, cwd=None):
    return subprocess.run([str(value) for value in arguments], capture_output=True, text=True, cwd=cwd)


def emit(source, output, extra=()):
    return execute(emitter, "--root", KEY, "--crate", "metadata_fixture", KEY,
                   source, source.name, output, *extra)


def inspect(source, dependency="-", name="metadata_fixture"):
    result = execute(probe, "analyze", source, "-", dependency, "--package",
                     "--crate-name", name, "--crate-key", KEY)
    assert result.returncode == 0, result.stderr
    return [line.split("\t") for line in result.stdout.splitlines()]


def identity(records, role):
    matches = [row[3:] for row in records if row[:3] == ["crate", role, "metadata_fixture"]]
    assert len(matches) == 1, records
    return matches[0]


output = work / "libdirect.rmeta"
result = emit(fixture, output)
assert result.returncode == 0, result.stderr
if output.read_bytes() != artifact.read_bytes():
    direct_strings = set(execute("strings", output).stdout.splitlines())
    action_strings = set(execute("strings", artifact).stdout.splitlines())
    print("direct-only metadata strings:", sorted(direct_strings - action_strings), flush=True)
    print("action-only metadata strings:", sorted(action_strings - direct_strings), flush=True)
assert output.read_bytes() == artifact.read_bytes(), "Bazel and direct metadata emission differ"
consumer = work / "consumer.rs"
consumer.write_text("pub fn identity(value: i32) -> i32 { renamed::identity(value) }\n")
expected = identity(inspect(fixture), "local")
assert identity(inspect(consumer, artifact, "metadata_consumer"), "foreign") == expected
assert identity(inspect(consumer, output, "metadata_consumer"), "foreign") == expected
relocated = work / "relocated"
relocated.mkdir()
copy = relocated / fixture.name
copy.write_bytes(fixture.read_bytes())
other_output = relocated / "librelocated.rmeta"
result = emit(copy, other_output)
assert result.returncode == 0, result.stderr
assert other_output.read_bytes() == artifact.read_bytes(), "metadata depends on physical source location"

# A shorter physical filename is a textual prefix even across extension dots.
# Give it a later logical name to defeat accidental logical-order correctness.
prefix_doc = work / "prefix"
prefix_doc.write_text("Prefix mapping documentation.\n")
prefix_outputs = []
for physical in ["prefix.rs", "unrelated.rs"]:
    source = work / physical
    source.write_text('#[doc = include_str!("prefix")]\npub fn identity(value: i32) -> i32 { value }\n')
    target = work / (physical + ".rmeta")
    result = execute(emitter, "--root", KEY, "--crate", "metadata_fixture", KEY,
                     source, "a.rs", target, "--input", prefix_doc, "z.md")
    assert result.returncode == 0, result.stderr
    prefix_outputs.append(target.read_bytes())
assert prefix_outputs[0] == prefix_outputs[1], "overlapping physical prefixes corrupted logical source identity"
prefix_cwd = work / "prefix.rs.build"
prefix_cwd.mkdir()
prefix_cwd_output = work / "libprefix_cwd.rmeta"
result = execute(emitter, "--root", KEY, "--crate", "metadata_fixture", KEY,
                 work / "prefix.rs", "a.rs", prefix_cwd_output,
                 "--input", prefix_doc, "z.md", cwd=prefix_cwd)
assert result.returncode == 0, result.stderr
assert prefix_cwd_output.read_bytes() == prefix_outputs[0], "file prefix corrupted normalized compiler cwd"

before = output.read_bytes()
assert emit(fixture, output).returncode != 0
assert output.read_bytes() == before
assert emit(fixture, work).returncode != 0
link = work / "liblink.rmeta"
link.symlink_to(work / "nonexistent")
assert emit(fixture, link).returncode != 0 and link.is_symlink()

bad = work / "bad.rs"
bad_output = work / "libbad.rmeta"
for body, diagnostic in [
    ("pub fn identity(value: i32) -> i32 { true }\n", "mismatched types"),
    ('#[doc = include_str!("docs.md")]\npub fn identity(value: i32) -> i32 { value }\n', "undeclared compiler file input"),
]:
    (work / "docs.md").write_text("Declared only in the positive case.\n")
    bad.write_text(body)
    result = emit(bad, bad_output)
    assert result.returncode != 0 and diagnostic in result.stderr, result.stderr
    assert not bad_output.exists()
    assert not list(work.glob(".polyrust-metadata-*"))
result = emit(bad, bad_output, ["--input", work / "docs.md", "docs.md"])
assert result.returncode == 0, result.stderr
assert bad_output.is_file()

rejected = work / "librejected.rmeta"
for extra in [["--extern", "ambient=/tmp/libambient.rmeta"], ["--sysroot", "/other"], ["-Aunsafe-code"]]:
    assert emit(fixture, rejected, extra).returncode != 0
    assert not rejected.exists()
extra = ["--dependency", "leaf", "leaf.key", "--crate", "leaf", "leaf.key",
         fixture, fixture.name, work / "libleaf.rmeta"]
result = emit(fixture, rejected, extra)
assert result.returncode != 0 and "declared dependency metadata" in result.stderr, result.stderr
assert not rejected.exists()
assert not list(work.glob(".polyrust-metadata-*"))
large_output = work / "liblarge_response.rmeta"
large_arguments = ["--root", KEY, "--crate", "metadata_fixture", KEY,
                   str(fixture), fixture.name, str(large_output)]
extra_files = work / "large_inputs"
extra_files.mkdir()
for index in range(800):
    extra_file = extra_files / str(index)
    extra_file.write_text("")
    large_arguments.extend(["--input", str(extra_file), f"docs/{index}/" + "a" * 3000])
response = work / "large.params"
response.write_text("\n".join(large_arguments) + "\n")
assert response.stat().st_size > 2 * 1024 * 1024
result = execute(emitter, "@" + str(response))
assert result.returncode == 0, result.stderr
assert large_output.is_file()
assert not list(work.glob(".polyrust-metadata-*"))
print("declared metadata: action/direct/relocated byte equality, compiler-loaded identity and no-replace failure contracts passed")
