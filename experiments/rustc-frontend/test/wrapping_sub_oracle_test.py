"""Borrow/overflow coverage, independent invariants and pinned native agreement."""
from pathlib import Path
import subprocess
import sys
from wrapping_sub_oracle import CASES, WIDTHS, cases, expected, faulty, inputs, limits, result, signed


def check():
    for width in WIDTHS:
        low, high = limits(width)
        known = [(low, 1, high), (high, -1, low), (low, low, 0), (high, high, 0),
                 (0, low, low), (0, high, -high), (low, high, 1), (high, low, -1),
                 (0, 1, -1), (1, 0, 1), (-1, 1, -2), (1, -1, 2)]
        for left, right, wanted in known:
            assert result(left, right, width) == wanted
        pairs = cases(width)
        inventory = set(pairs)
        assert len(pairs) == len(inventory)
        for bit in range(width):
            boundary = signed(1 << bit, width)
            assert (boundary, 1) in inventory and (1, boundary) in inventory
            assert result(boundary, 1, width) == signed(boundary - 1, width)
        assert any(a - b > high for a, b in pairs)
        assert any(a - b < low for a, b in pairs)
        assert any(a != b and result(a, b, width) != result(b, a, width) for a, b in pairs)
        for fault in ["saturating", "add", "reverse", "borrowless", "narrow"]:
            assert any(result(a, b, width) != faulty(a, b, width, fault) for a, b in pairs), (width, fault)
        for left, right in pairs:
            value = result(left, right, width)
            assert low <= value <= high
            assert (value - left + right) % (1 << width) == 0
            assert result(left, 0, width) == left
            assert result(left, left, width) == 0
            assert result(left, right, width) == signed(-result(right, left, width), width)
    # Pin the reviewed shared corpus so accidental shrinkage requires attention.
    assert len(CASES) == len(set(CASES)) == 15_790


def main():
    check()
    truth, data = expected(), inputs()
    assert len(sys.argv) == 3
    for reference in map(Path, sys.argv[1:]):
        native = subprocess.run([str(reference.resolve())], input=data, text=True,
                                capture_output=True, timeout=90)
        assert native.returncode == 0 and native.stderr == "", native.stderr[:2000]
        assert native.stdout == truth, reference
    print(f"{len(CASES)} signed 32/64-bit differences match unbounded modular truth and "
          "native Rust with overflow checks on/off; five fault families detected")


if __name__ == "__main__":
    main()
