"""Instrument only the two original Rust producers for a native trace oracle."""
from pathlib import Path
import sys

source, destination = map(Path, sys.argv[1:])
original = source.read_text()
traced = original
for name, marker in [("left", "A"), ("right", "B")]:
    needle = f"pub fn {name}(value: f64) -> f64 {{\n    identity(value)"
    assert traced.count(needle) == 1
    traced = traced.replace(needle, needle.replace("    identity", f'    eprint!("{marker}");\n    identity'))
assert traced.replace('    eprint!("A");\n', '').replace('    eprint!("B");\n', '') == original
destination.write_text(traced)
