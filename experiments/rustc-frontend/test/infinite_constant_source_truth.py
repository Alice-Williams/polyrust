"""Integer-only source truth, independent of target mappings and float arithmetic."""
from infinite_constant_oracle import EXPRESSIONS, INFINITY, SIGN

CONSTANTS = {name.upper(): sign | INFINITY for name, sign in EXPRESSIONS}
# An isolated producer sign is the actual Bazel invalidation control.
CONSTANTS["BITS_POSITIVE"] = INFINITY
READS = {name.lower(): bits for name, bits in CONSTANTS.items()}
READS.update(other_negative=SIGN | INFINITY, other_same=INFINITY, alias=INFINITY,
             own=SIGN | INFINITY, private=INFINITY, local=SIGN | INFINITY,
             unused=SIGN | INFINITY, inherent=SIGN | INFINITY)
READS.update(absolute=INFINITY, negation=SIGN | INFINITY, addition=SIGN | INFINITY,
             subtraction=INFINITY, multiplication=INFINITY, division=0,
             negative_zero=SIGN, nan=0, remainder=0x3ff0000000000000,
             truncation=SIGN | INFINITY, comparison=0x3ff0000000000000)


def text(values):
    return "".join(f"{value:016x}\n" for value in values)


def reject(check):
    try:
        check()
    except (AssertionError, KeyError):
        return
    raise AssertionError("fault escaped the proof")
