"""Independent packet truth for original multi-owner character source fixtures."""
import struct

from character_oracle import BOUNDARIES, MAXIMUM, PAIRS, SCALAR_COUNT, flags, scalar

INPUT = struct.Struct("<IIi")
OUTPUT = struct.Struct("<9I")
MARKERS = (-2147483648, -1, 0, 1, 2147483647)
LITERAL_PREFIX = b"".join(struct.pack("<I", value) for value in BOUNDARIES)


def cases(full=True):
    values = (value for value in range(MAXIMUM + 1) if scalar(value)) if full else BOUNDARIES
    for index, value in enumerate(values):
        yield value, 0, MARKERS[index % len(MARKERS)]
    for index, (left, right) in enumerate(PAIRS):
        yield left, right, MARKERS[index % len(MARKERS)]


def row(left, right, marker):
    assert scalar(left) and scalar(right)
    assert -(1 << 31) <= marker < 1 << 31
    # Identity/local/alias, both selections, mixed record's Char and I32,
    # six direct comparison bits and nested comparison through original owners.
    return (left, left, left, left, right, left, marker & 0xffffffff,
            flags(left, right), int(left < right))


def corpus(full=True):
    inputs, outputs = bytearray(), bytearray(LITERAL_PREFIX)
    count = 0
    for left, right, marker in cases(full):
        inputs.extend(INPUT.pack(left, right, marker))
        outputs.extend(OUTPUT.pack(*row(left, right, marker)))
        count += 1
    assert count == (SCALAR_COUNT if full else len(BOUNDARIES)) + len(PAIRS)
    assert len(inputs) == count * INPUT.size
    assert len(outputs) == len(LITERAL_PREFIX) + count * OUTPUT.size
    return bytes(inputs), bytes(outputs), count


def trace(left, right, marker):
    identity = f"I:{left};"
    other = f"I:{right};"
    field = f"M:{marker};"
    # Root identity/local/alias, eager arguments to both selections, then
    # record-character (marker before char) and record-marker (char before marker),
    # followed by nested less-than's two independently evaluated operands.
    return (identity * 3 + (identity + other) * 2
            + identity + field + identity
            + identity * 2 + field
            + identity + other).encode()
