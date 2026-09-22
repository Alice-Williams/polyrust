"""Observe actual source calls while preserving each original body byte-for-byte."""
from pathlib import Path
import sys

source, destination = map(Path, sys.argv[1:])
original = source.read_text()
instrumented = original
insertions = [
    ("pub fn identity(value: char) -> char {\n", '    eprint!("I:{};", u32::from(value));\n'),
    ("fn hidden_marker(value: i32) -> i32 {\n", '    eprint!("M:{value};");\n'),
]
for needle, trace in insertions:
    assert instrumented.count(needle) == 1
    instrumented = instrumented.replace(needle, needle + trace)
restored = instrumented
for _, trace in insertions:
    assert restored.count(trace) == 1
    restored = restored.replace(trace, "")
assert restored == original and instrumented != original
destination.write_text(instrumented)
