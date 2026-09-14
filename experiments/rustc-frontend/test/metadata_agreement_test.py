"""Actual emitted/loaded rustc identities, including same-signature mutations."""
import os
from pathlib import Path
import subprocess
import sys

probe = Path(sys.argv[1]).resolve()
work = Path(os.environ["TEST_TMPDIR"]) / "metadata-agreement"
work.mkdir()


def invoke(mode, source, metadata, dependency="-", name="metadata_fixture", key="probe.v1", inputs=()):
    arguments = [probe, mode, source, metadata, dependency, "--package",
                 "--crate-name", name, "--crate-key", key]
    for path in inputs:
        arguments.extend(["--input", path])
    return subprocess.run([str(value) for value in arguments], capture_output=True, text=True)


def success(*args, **kwargs):
    result = invoke(*args, **kwargs)
    assert result.returncode == 0, result.stderr
    return [line.split("\t") for line in result.stdout.splitlines()]


def record(records, kind, role, name):
    matches = [row[3:] for row in records if row[:3] == [kind, role, name]]
    assert len(matches) == 1, records
    return matches[0]


source = work / "dependency.rs"
source.write_text('/// First documentation.\npub fn identity(value: i32) -> i32 { value }\n')
consumer = work / "consumer.rs"
consumer.write_text('use renamed::identity as local_alias;\npub fn identity(value: i32) -> i32 { local_alias(value) }\n')
metadata = work / "libmetadata_fixture.rmeta"
local = success("analyze", source, "-")
emitted = success("emit", source, metadata)
assert metadata.is_file() and metadata.stat().st_size > 0
assert local == emitted, (local, emitted)
assert local == success("repeat", source, "-")
loaded = success("repeat", consumer, "-", metadata, name="metadata_consumer")
assert record(local, "crate", "local", "metadata_fixture") == record(loaded, "crate", "foreign", "metadata_fixture")
assert record(local, "function", "local", "identity") == record(loaded, "function", "foreign", "identity")
sequential = success("sequence", source, consumer, metadata)
for kind, name in [("crate", "metadata_fixture"), ("function", "identity")]:
    assert record(sequential, kind, "local", name) == record(sequential, kind, "foreign", name)

for label, text in [
    ("body", '/// First documentation.\npub fn identity(value: i32) -> i32 { -value }\n'),
    ("docs", '/// Different documentation.\npub fn identity(value: i32) -> i32 { value }\n'),
]:
    source.write_text(text)
    changed = success("analyze", source, "-")
    assert record(changed, "function", "local", "identity") == record(local, "function", "local", "identity")
    original_crate = record(local, "crate", "local", "metadata_fixture")
    changed_crate = record(changed, "crate", "local", "metadata_fixture")
    assert changed_crate[0] == original_crate[0], label
    assert changed_crate[1] != original_crate[1], (label, local, changed)
    assert record(loaded, "crate", "foreign", "metadata_fixture") != changed_crate
    changed_metadata = work / f"libmetadata_fixture-{label}.rmeta"
    assert success("emit", source, changed_metadata) == changed
    changed_loaded = success("repeat", consumer, "-", changed_metadata, name="metadata_consumer")
    assert record(changed_loaded, "crate", "foreign", "metadata_fixture") == changed_crate
    assert record(changed_loaded, "function", "foreign", "identity") == record(local, "function", "local", "identity")

source.write_text('/// First documentation.\npub fn identity(value: i32) -> i32 { value }\n')
assert success("analyze", source, "-") == local
different_key = success("analyze", source, "-", key="probe.v2")
assert record(different_key, "crate", "local", "metadata_fixture")[0] != record(local, "crate", "local", "metadata_fixture")[0]

relocated = work / "relocated.rs"
relocated.write_bytes(source.read_bytes())
relocation = success("analyze", relocated, "-")
assert record(relocation, "function", "local", "identity") == record(local, "function", "local", "identity")
assert record(relocation, "crate", "local", "metadata_fixture") != record(local, "crate", "local", "metadata_fixture")
another_directory = work / "another sandbox with spaces"
another_directory.mkdir()
same_logical_source = another_directory / source.name
same_logical_source.write_bytes(source.read_bytes())
assert success("analyze", same_logical_source, "-") == local
assert success("unmapped", same_logical_source, "-") != success("unmapped", source, "-")
print("content identity: logical filename-sensitive, stable across explicitly remapped source directories")

doc = work / "docs.md"
doc.write_text("Included documentation.\n")
source.write_text('#[doc = include_str!("docs.md")]\npub fn identity(value: i32) -> i32 { value }\n')
included = success("analyze", source, "-", inputs=[doc])
assert invoke("analyze", source, "-").returncode != 0
doc.write_text("Changed included documentation.\n")
assert success("analyze", source, "-", inputs=[doc]) != included

module = work / "values.rs"
module.write_text('pub(crate) fn helper(value: i32) -> i32 { value }\n')
source.write_text('mod values;\n#[doc = include_str!("docs.md")]\npub fn identity(value: i32) -> i32 { values::helper(value) }\n')
multifile = success("analyze", source, "-", inputs=[doc, module])
assert success("emit", source, metadata, inputs=[module, doc]) == multifile
multifile_loaded = success("analyze", consumer, "-", metadata, name="metadata_consumer")
assert record(multifile_loaded, "crate", "foreign", "metadata_fixture") == record(multifile, "crate", "local", "metadata_fixture")
same_logical_source.write_bytes(source.read_bytes())
relocated_doc = another_directory / doc.name
relocated_module = another_directory / module.name
relocated_doc.write_bytes(doc.read_bytes())
relocated_module.write_bytes(module.read_bytes())
assert success("repeat", same_logical_source, "-", inputs=[relocated_module, relocated_doc]) == multifile
missing_module = invoke("analyze", source, "-", inputs=[doc])
assert missing_module.returncode != 0 and "undeclared compiler file input" in missing_module.stderr
module.write_text('pub(crate) fn helper(value: i32) -> i32 { -value }\n')
assert success("analyze", source, "-", inputs=[doc, module]) != multifile
print("per-file mappings: root/module/doc relocation, input-order independence and undeclared module rejection passed")

source.write_text('fn hidden(value: i32) -> i32 { value }\npub fn identity(value: i32) -> i32 { hidden(value) }\n')
success("emit", source, metadata)
consumer.write_text('pub fn identity(value: i32) -> i32 { renamed::hidden(value) }\n')
private = invoke("analyze", consumer, "-", metadata, name="metadata_consumer")
assert private.returncode != 0 and "private" in private.stderr, private.stderr
source.write_text('pub fn identity(value: i32) -> i32 { true }\n')
invalid = invoke("analyze", source, "-")
assert invalid.returncode != 0 and "mismatched types" in invalid.stderr, invalid.stderr
print("metadata agreement: local/emitted/loaded identity, body/docs/key mutations, aliases, sequential analysis and compiler rejection passed")
