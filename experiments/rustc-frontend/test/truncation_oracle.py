"""Integer-only rounding/category oracle; no floating arithmetic."""
from binary64_oracle import VALUES as BASE, SIGN, MAGNITUDE, observed

OPERATIONS = ["direct", "associated", "local", "imported", "forwarded", "shared", "record", "composed"]
LITERALS = ["positive_zero", "negative_zero"]
VALUES = list(BASE)
for exponent in [0, 1, 1022, 1023, 1024, 1074, 1075, 1076, 2046]:
    for fraction in [0, 1, (1 << 51) - 1, 1 << 51, (1 << 52) - 1]:
        for sign in [0, SIGN]:
            VALUES.append(sign | (exponent << 52) | fraction)
VALUES = list(dict.fromkeys(VALUES))

def rounding(bits, mode="trunc"):
    sign = bits & SIGN
    exponent = ((bits >> 52) & 0x7ff) - 1023
    if exponent >= 52:
        return bits
    truncated = sign if exponent < 0 else bits & ~((1 << (52 - exponent)) - 1)
    if mode == "zero_sign":
        return 0 if truncated & MAGNITUDE == 0 else truncated
    if truncated == bits or mode == "trunc":
        return truncated
    assert mode in ["floor", "ceil"]
    away = (mode == "floor" and sign != 0) or (mode == "ceil" and sign == 0)
    if not away:
        return truncated
    return sign | 0x3ff0000000000000 if exponent < 0 else truncated + (1 << (52 - exponent))

def expected():
    return "\n".join([observed(0), observed(SIGN)] +
                     [observed(rounding(bits ^ SIGN if operation == "composed" else bits))
                      for bits in VALUES for operation in OPERATIONS]) + "\n"

def traces(variant):
    if variant == "plain":
        return ""
    return {"dropped": "BAB", "duplicated": "AABAB",
            "ordinary_builtin": "AA", "ordinary_duplicated": "ABBABB"}.get(variant, "ABAB") * len(VALUES)
