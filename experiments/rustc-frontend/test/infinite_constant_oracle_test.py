"""Exact native agreement, pinned category inventory and hostile classifiers."""
from collections import Counter
from pathlib import Path
import subprocess
import sys

from infinite_constant_oracle import (
    CASES, EXPRESSIONS, FRACTION, INFINITY, MAX_FINITE, SIGN,
    classify, expected, faulty, inputs,
)


def rejected(check):
    try:
        check()
    except AssertionError:
        return
    raise AssertionError("invalid oracle input was accepted")


def check():
    for invalid in [-1, 1 << 64]:
        rejected(lambda: classify(invalid))
    for invalid in [-1, 1 << 64, 0, MAX_FINITE, INFINITY | 1]:
        for fault in ["sign_loss", "finite_clamp", "zero"]:
            rejected(lambda: faulty(invalid, fault))
    rejected(lambda: faulty(INFINITY, "unknown fault"))
    assert len(EXPRESSIONS) == len({name for name, _ in EXPRESSIONS}) == 20
    assert Counter(sign for _, sign in EXPRESSIONS) == {0: 10, SIGN: 10}
    assert len(CASES) == len(set(CASES)) == 24_934
    assert Counter(map(classify, CASES)) == {
        "finite": 24_566, "nan": 366, "positive": 1, "negative": 1,
    }
    assert {0, SIGN, 1, SIGN | 1, MAX_FINITE, SIGN | MAX_FINITE,
            INFINITY, SIGN | INFINITY} <= set(CASES)
    for sign in [0, SIGN]:
        for bit in range(52):
            assert sign | INFINITY | (1 << bit) in CASES
            assert classify(sign | INFINITY | (1 << bit)) == "nan"
        assert classify(sign | INFINITY | FRACTION) == "nan"
    for fault in ["sign_loss", "finite_clamp", "zero"]:
        assert any(faulty(sign | INFINITY, fault) != sign | INFINITY
                   for _, sign in EXPRESSIONS)
    assert faulty(SIGN | INFINITY, "sign_loss") == INFINITY
    assert faulty(INFINITY, "finite_clamp") == MAX_FINITE
    assert faulty(SIGN | INFINITY, "finite_clamp") == SIGN | MAX_FINITE
    # Each incorrect classification rule must be distinguished by the corpus.
    faults = [
        lambda bits: "positive" if bits & INFINITY == INFINITY else "finite",
        lambda bits: "positive" if classify(bits) == "negative" else classify(bits),
        lambda bits: "positive" if bits == MAX_FINITE else classify(bits),
    ]
    for fault in faults:
        assert any(fault(bits) != classify(bits) for bits in CASES)


def main():
    check()
    assert len(sys.argv) == 3
    truth = expected()
    for reference in map(Path, sys.argv[1:]):
        native = subprocess.run([str(reference.resolve())], input=inputs(), text=True,
                                capture_output=True, timeout=90)
        assert native.returncode == 0 and native.stderr == "", native.stderr[:2000]
        assert native.stdout == truth, reference
    print("20 computed infinity constants and 24,934 classification patterns agree with "
          "Rust O0/checks-on + O2/checks-off; three actual value faults and three "
          "classification fault families detected")


if __name__ == "__main__":
    main()
