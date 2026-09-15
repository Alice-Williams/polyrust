"""Pinned compiler observations; no conditional ownership admission."""
from pathlib import Path
import subprocess
import sys
import tempfile

probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
result = subprocess.run([str(probe), str(fixture)], capture_output=True,
                        text=True, check=False, timeout=60)
assert result.returncode == 0, result.stderr
assert "conditional ownership observation inventory passed" in result.stdout
assert result.stdout.count("exact source, typed events, flags, cleanup and exit observed") == 12
print(result.stdout)

original = fixture.read_text()
success = "conditional ownership observation inventory passed"
with tempfile.TemporaryDirectory(prefix="conditional-owned-") as directory:
    source = Path(directory) / "fixture.rs"
    for name, body, error in [
        ("uninitialized", "let record; if flag { record = Pair { first: Box::new(a), second: Box::new(b) }; } drop(record);", "E0381"),
        ("partial_reuse", "let record = Pair { first: Box::new(a), second: Box::new(b) }; if flag { let taken = record.first; } drop(record);", "E0382"),
        ("duplicate_field", "let record = Pair { first: Box::new(a), second: Box::new(b) }; let one = record.first; let two = record.first;", "E0382"),
    ]:
        source.write_text(original + "\nfn invalid(flag: bool, a: i32, b: i32) { " + body + " }\n")
        result = subprocess.run([str(probe), str(source)], capture_output=True, text=True, check=False, timeout=60)
        assert result.returncode != 0 and error in result.stderr, (name, result.stderr)
        assert success not in result.stdout
    controls = {
        "guard": (original.replace("if flag {", "if !flag {", 1), "local path"),
        "constant_guard": (original.replace("if flag {", "if true {", 1), "local path"),
        "scalar_anchor": (original.replace("Box::new(a)", "Box::new(b)", 1), "scalar parameter anchor"),
        "constructor": (original.replace("Box::new(a)", "Box::from(a)", 1), "standard Box constructor"),
        "initializer_order": (original.replace("record = Pair { first, second };", "record = Pair { second, first };", 1), "initializer source order"),
        "field": (original.replace("let taken = record.first;", "let taken = record.second;", 1), "selected source field"),
        "read": (original.replace("return *taken;", "return *keep;", 1), "selected source owner"),
        "mutable_parameter": (original.replace("initialize(flag:", "initialize(mut flag:", 1), "immutable simple binding"),
        "mutable_binding": (original.replace("let first =", "let mut first =", 1), "immutable simple binding"),
        "representation": (original.replace("struct Pair {", "#[repr(C)]\nstruct Pair {", 1), "default record representation"),
        "generic_record": (original.replace("struct Pair {", "struct Pair<T> {").replace("first: Box<i32>,", "first: Box<T>,"), ""),
        "extra_statement": (original.replace("if flag {", "if flag { let extra = 1;", 1), ""),
        "nominal": (original.replace("record = Pair { first, second };", "record = Alternate { first, second };", 1) + "\nstruct Alternate { first: Box<i32>, second: Box<i32> }\n", "one canonical Pair type"),
        "inventory": (original.replace("pub fn early_second(", "pub fn changed("), "unexpected fixture"),
        "empty": ("// Empty source must not satisfy an observation inventory.\n", ""),
    }
    for name, (text, marker) in controls.items():
        assert text != original, name
        source.write_text(text)
        result = subprocess.run([str(probe), str(source)], capture_output=True, text=True, check=False, timeout=60)
        assert result.returncode != 0 and success not in result.stdout, name
        assert "panicked at" in result.stderr and "error[E" not in result.stderr, (name, result.stderr)
        assert marker in result.stderr, (name, marker, result.stderr)
print("three invalid Rust and fifteen valid-source conditional observation controls passed")
