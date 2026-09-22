"""Instrument the original operand producer; preserve its body exactly."""
from pathlib import Path
import sys

source, destination = map(Path, sys.argv[1:])
original = source.read_text()
needle = "pub fn input(value: i32) -> i32 {\n"
assert original.count(needle) == 1
trace = '    eprint!("A");\n'
traced = original.replace(needle, needle + trace)
assert traced.replace(trace, "") == original
destination.write_text(traced)
