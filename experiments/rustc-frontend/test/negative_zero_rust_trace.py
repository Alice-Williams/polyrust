"""Instrument one private original producer, without changing the source model."""
from pathlib import Path
import sys

source, destination = map(Path, sys.argv[1:])
original = source.read_text()
needle = "fn reciprocal(value: f64) -> f64 {\n"
assert original.count(needle) == 1
traced = original.replace(needle, needle + '    eprint!("R");\n')
assert traced.replace('    eprint!("R");\n', '') == original
destination.write_text(traced)
