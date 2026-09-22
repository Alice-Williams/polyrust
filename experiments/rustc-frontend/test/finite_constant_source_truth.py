"""Independent bit expectations for the fixed original-source fixture."""
from finite_constant_oracle import COMPUTED, SIGN

CONSTANTS = {
    "POSITIVE_ZERO": 0,
    "NEGATIVE_ZERO": SIGN,
    "MIN_SUBNORMAL": 1,
    "MAX_SUBNORMAL": (1 << 52) - 1,
    "MIN_NORMAL": 1 << 52,
    "MAX_FINITE": 0x7fefffffffffffff,
    "NEGATIVE_MAX": 0xffefffffffffffff,
    "ONE_ULP": 0x3ff0000000000001,
    "TENTH": COMPUTED[1],
    "HALF_INTEGER": COMPUTED[2],
    "HALF_ULP": COMPUTED[3],
    "SUM": COMPUTED[4],
    "THIRD": COMPUTED[5],
    "HALF_NORMAL": COMPUTED[6],
    "TWO_SUBNORMALS": COMPUTED[7],
    "HALF_MAX": COMPUTED[8],
    "ZERO_PRODUCT": COMPUTED[9],
    "INDEXED": 0x3fe0000000000000,
    "FORWARD": COMPUTED[1],
    "SAME_VALUE": COMPUTED[1],
}
READS = {name.lower(): value for name, value in CONSTANTS.items()}
READS.update(other_tenth=COMPUTED[1], alias=COMPUTED[1], own=CONSTANTS["INDEXED"],
             private=SIGN, local=COMPUTED[5], unused=CONSTANTS["INDEXED"], inherent=1)
READS.update(absolute=0x3fe0000000000000, arithmetic=0x3fe8000000000000,
             negation=0xbfe0000000000000, nan=0, remainder=0, truncation=0)
READS["other_same"] = CONSTANTS["TENTH"]


def text(values):
    return "".join(f"{value:016x}\n" for value in values)


def reject(check):
    try:
        check()
    except (AssertionError, KeyError):
        return
    raise AssertionError("fault escaped the proof")
