"""Integer-only scalar identity/order; no Unicode database or target imports."""
import struct

MAXIMUM = 0x10ffff
SURROGATE_START = 0xd800
SURROGATE_END = 0xdfff
SCALAR_COUNT = 1_112_064
PACKET = struct.Struct("<III")
RESULT = struct.Struct("<IIIIII")


def scalar(value):
    assert type(value) is int and 0 <= value <= 0xffffffff
    return value <= MAXIMUM and not SURROGATE_START <= value <= SURROGATE_END


def flags(left, right):
    return sum(int(test) << bit for bit, test in enumerate(
        [left == right, left != right, left < right,
         left <= right, left > right, left >= right]))


def utf16(value):
    assert scalar(value)
    if value < 0x10000:
        return (value,)
    reduced = value - 0x10000
    return (0xd800 + (reduced >> 10), 0xdc00 + (reduced & 1023))


def from_ordinal(ordinal):
    assert type(ordinal) is int and 0 <= ordinal < SCALAR_COUNT
    return ordinal if ordinal < SURROGATE_START else ordinal + 2048


def outside():
    values = {MAXIMUM + 1, MAXIMUM + 2, 0x1fffff, 0x7fffffff, 0x80000000, 0xffffffff}
    values.update(1 << bit for bit in range(21, 32))
    state = 0x91a547d0
    for _ in range(64):
        state = (1664525 * state + 1013904223) % (1 << 32)
        if state > MAXIMUM:
            values.add(state)
    return tuple(sorted(values))


OUTSIDE = outside()
BOUNDARIES = (0, 1, 0x7f, 0x80, 0xff, 0x100, 0x378, 0x7ff, 0x800,
              0xd7ff, 0xe000, 0xfdd0, 0xfffe, 0xffff, 0x10000, 0x1f980,
              0xf0000, 0x10fffe, MAXIMUM)


def pairs():
    values = [(left, right) for left in BOUNDARIES for right in BOUNDARIES]
    state = 0x15791abc
    for index in range(4096):
        state = (1664525 * state + 1013904223) % (1 << 32)
        left = from_ordinal(state % SCALAR_COUNT)
        right = left if index % 7 == 0 else from_ordinal((state * 65537) % SCALAR_COUNT)
        values.append((left, right))
    return tuple(dict.fromkeys(values))


PAIRS = pairs()


def packets():
    for value in range(MAXIMUM + 1):
        yield 0, value, 0
    for value in OUTSIDE:
        yield 0, value, 0
    for left, right in PAIRS:
        yield 1, left, right


def expected(tag, left, right):
    assert type(tag) is int and tag in (0, 1)
    if tag == 0:
        assert type(right) is int and right == 0
        valid = scalar(left)
        return (int(valid), left if valid else 0, left & 0xffff, left & 0xff,
                int(valid and left <= 0xffff), int(left <= MAXIMUM))
    assert tag == 1 and scalar(left) and scalar(right)
    return (flags(left, right), flags(right, left), flags(utf16(left), utf16(right)),
            flags(left & 0xff, right & 0xff), left, right)


def inputs():
    return b"".join(PACKET.pack(*row) for row in packets())
