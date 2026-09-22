"""Integer-only character constant values; no target or Unicode database."""
from character_oracle import BOUNDARIES, SCALAR_COUNT, from_ordinal

INTERIOR = tuple(from_ordinal((index * 65537 + 17) % SCALAR_COUNT) for index in range(4096))
CONTEXTS = (0x1f980, 0x1f980, 0xfdd0, 0x10ffff, 0xe000, 0x10000,
            0xd7ff, 0x27, 0x5c, 0xa, 0x378, 0x1f980)
FAULTS = ("byte", "code_unit", "replacement", "changed")


def constant(value):
    if type(value) is not int or not 0 <= value <= 0x10ffff or 0xd800 <= value <= 0xdfff:
        raise ValueError("constant is not a Unicode scalar")
    return value


def expected_values():
    return tuple(constant(value) for value in (*BOUNDARIES, *INTERIOR, *CONTEXTS))


def faulty(value, fault):
    constant(value)
    if fault == "byte":
        return value & 255
    if fault == "code_unit":
        return value & 65535
    if fault == "replacement":
        return value if value < 128 else 0xfffd
    if fault == "changed":
        return value ^ 1
    raise ValueError("unknown constant fault")


def expected_output():
    values = expected_values()
    rows = [f"character-constants-v1 {len(values)}"]
    rows += [" ".join(f"{value:08x}" for value in (original, *[faulty(original, fault)
             for fault in FAULTS])) for original in values]
    return "\n".join([*rows, *["invalid 0"] * 4]) + "\n"


def verify(output):
    if output != expected_output():
        raise ValueError("native constant observations differ from independent truth")
