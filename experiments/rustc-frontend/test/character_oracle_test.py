"""Full scalar interval and actual native value/admission/ordering faults."""
from pathlib import Path
import subprocess
import sys

from character_oracle import (
    BOUNDARIES, MAXIMUM, OUTSIDE, PACKET, PAIRS, RESULT, SCALAR_COUNT,
    expected, flags, from_ordinal, inputs, packets, scalar, utf16,
)


def rejects(check):
    try:
        check()
    except AssertionError:
        return
    raise AssertionError("invalid oracle input was accepted")


def check_oracle():
    for value in [-1, 1 << 32, True, 1.5]:
        rejects(lambda: scalar(value))
    for value in [-1, SCALAR_COUNT, True]:
        rejects(lambda: from_ordinal(value))
    for value in [0xd800, 0xdfff, MAXIMUM + 1]:
        rejects(lambda: utf16(value))
    rejects(lambda: expected(2, 0, 0))
    rejects(lambda: expected(True, 0, 0))
    rejects(lambda: expected(0, 0, False))
    rejects(lambda: expected(1, 0xd800, 0))
    rejects(lambda: expected(0, 0, 1))
    assert len(OUTSIDE) == 80 and all(value > MAXIMUM for value in OUTSIDE)
    assert all(not scalar(value) for value in OUTSIDE)
    assert tuple(sorted(set(OUTSIDE))) == OUTSIDE
    assert sum(scalar(value) for value in range(MAXIMUM + 1)) == SCALAR_COUNT
    assert sum(not scalar(value) for value in range(MAXIMUM + 1)) == 2048
    assert from_ordinal(0) == 0 and from_ordinal(SCALAR_COUNT - 1) == MAXIMUM
    assert from_ordinal(0xd7ff) == 0xd7ff and from_ordinal(0xd800) == 0xe000
    assert all(scalar(value) for value in BOUNDARIES)
    assert len(PAIRS) == len(set(PAIRS)) == 4453
    assert (0xe000, 0x10000) in PAIRS and (0, 0x100) in PAIRS
    assert flags(0xe000, 0x10000) != flags(utf16(0xe000), utf16(0x10000))
    assert utf16(0x10000) == (0xd800, 0xdc00)
    assert utf16(MAXIMUM) == (0xdbff, 0xdfff)


def compare(output):
    count = MAXIMUM + 1 + len(OUTSIDE) + len(PAIRS)
    assert len(output) == count * RESULT.size
    detected = [False] * 7
    for packet, row in zip(packets(), RESULT.iter_unpack(output), strict=True):
        assert row == expected(*packet), (packet, row, expected(*packet))
        tag, left, _ = packet
        if tag == 0:
            if row[0]:
                detected[0] |= row[2] != row[1]
                detected[1] |= row[3] != row[1]
            detected[2] |= row[4] != row[0]
            detected[3] |= row[5] != row[0]
        else:
            for fault in range(3):
                detected[4 + fault] |= row[1 + fault] != row[0]
        if tag == 0 and 0xd800 <= left <= 0xdfff:
            assert row[0] == 0
    assert all(detected), detected


def main():
    check_oracle()
    assert len(sys.argv) == 3
    data = inputs()
    for reference in map(Path, sys.argv[1:]):
        command = [str(reference.resolve())]
        result = subprocess.run(command, input=data, capture_output=True, timeout=120)
        assert result.returncode == 0 and not result.stderr, result.stderr[:2000]
        compare(result.stdout)
        # The fixed-width protocol must reject, not silently ignore, trailing bytes.
        for invalid in [b"\x00", PACKET.pack(2, 0, 0), PACKET.pack(0, 0, 1),
                        PACKET.pack(1, 0xd800, 0), PACKET.pack(1, 0, MAXIMUM + 1)]:
            malformed = subprocess.run(command, input=invalid, capture_output=True, timeout=10)
            assert malformed.returncode != 0
    print(f"{SCALAR_COUNT:,} scalars and all 2,048 surrogates; {len(OUTSIDE)} outside u32 probes; "
          f"{len(PAIRS)} comparison pairs; Rust O0/checks-on + O2/checks-off; "
          "seven actual value/admission/order fault families detected")


if __name__ == "__main__":
    main()
