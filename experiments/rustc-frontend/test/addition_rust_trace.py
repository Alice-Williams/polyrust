"""Instrument actual Rust producers without altering their value computation."""
from pathlib import Path
import sys

source, destination = map(Path, sys.argv[1:])
original = source.read_text()
traced = original
for width, markers in [(32, "AB"), (64, "CD")]:
    for side, marker in zip(["left", "right"], markers, strict=True):
        needle = f"pub fn {side}{width}(value: i{width}) -> i{width} {{\n"
        assert traced.count(needle) == 1
        traced = traced.replace(needle, needle + f'    eprint!("{marker}");\n')
restored = traced
for marker in "ABCD":
    restored = restored.replace(f'    eprint!("{marker}");\n', '')
assert restored == original
destination.write_text(traced)
