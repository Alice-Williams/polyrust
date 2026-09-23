"""Pinned Rust agreement, exact success/error protocol and actual faulty programs."""
from pathlib import Path
import subprocess
import sys

from narrowing_oracle import CASES, FAULTS, HIGH, LOW, MAX, MIN, decode, faulty, inputs, result


def rejects(call):
    try:
        call()
    except (AssertionError, ValueError):
        return
    raise AssertionError("invalid oracle/protocol input accepted")


def check():
    inventory = set(CASES)
    assert len(CASES) == len(inventory) == 74_389
    assert sum(result(value) is not None for value in CASES) == 69_853
    assert set(range(-(1 << 15), 1 << 15)) <= inventory
    for edge in [MIN, LOW, HIGH, MAX]:
        assert {v for v in range(edge - 32, edge + 33) if MIN <= v <= MAX} <= inventory
    for bit in range(64):
        for sign in [-1, 1]:
            for delta in [-2, -1, 0, 1, 2]:
                value = sign * (1 << bit) + delta
                if MIN <= value <= MAX:
                    assert value in inventory
    assert [result(v) for v in [MIN, LOW - 1, LOW, -1, 0, 1, HIGH, HIGH + 1, MAX]] == [
        None, None, LOW, -1, 0, 1, HIGH, None, None]
    for invalid in [MIN - 1, MAX + 1, True, 0.0, "1"]:
        rejects(lambda: result(invalid))
    for fault in FAULTS:
        assert any(faulty(value, fault) != result(value) for value in CASES), fault
    for invalid in ["", "err err", "ok:0\n", "err err extra\n", "ok:01 err\n",
                    "ok:-0 err\n", "ok:+1 err\n", "ok:1.0 err\n", "ok:NaN err\n",
                    f"ok:{LOW - 1} err\n", f"ok:{HIGH + 1} err\n", "err err\n\n",
                    "err  err\n", "err err \n", "bad err\n", "err err\nerr err\n",
                    "err err\r\n", "err err\r"]:
        rejects(lambda: decode(invalid, 1))
    assert decode("ok:0 err\n", 1) == [(0, None)]


def main():
    check()
    assert len(sys.argv) == 3
    truth = [result(value) for value in CASES]
    for reference in map(Path, sys.argv[1:]):
        executable = str(reference.resolve())
        for fault in [None, *FAULTS]:
            native = subprocess.run([executable, *([fault] if fault else [])],
                                    input=inputs(), text=True, capture_output=True, timeout=90)
            assert native.returncode == 0 and not native.stderr, native.stderr[:2000]
            rows = decode(native.stdout, len(CASES))
            assert [row[1] for row in rows] == truth
            observed = [row[0] for row in rows]
            expected = [faulty(value, fault) for value in CASES] if fault else truth
            assert observed == expected, (reference, fault)
            assert (observed == truth) == (fault is None), fault
        for invalid in [str(MIN - 1), str(MAX + 1), "", "1.0", "NaN", " 1", "1 2"]:
            native = subprocess.run([executable], input=invalid + "\n", text=True,
                                    capture_output=True, timeout=30)
            assert native.returncode != 0 and native.stdout == "", (invalid, native)
        unknown = subprocess.run([executable, "unknown"], input="", text=True,
                                 capture_output=True, timeout=30)
        assert unknown.returncode != 0 and unknown.stdout == ""
    print(f"{len(CASES)} exact i64 inputs; TryFrom/TryInto at both profiles; "
          "six executable fault families; strict result protocol and invalid input rejection")


if __name__ == "__main__":
    main()
