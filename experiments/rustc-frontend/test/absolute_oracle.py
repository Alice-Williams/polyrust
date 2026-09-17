"""Exact non-NaN magnitude and NaN category, derived only with integer masks."""
from binary64_oracle import VALUES, MAGNITUDE, observed

OPERATIONS = ["direct", "associated", "local", "imported", "forwarded", "shared", "record", "composed"]
LITERALS = ["positive_zero", "negative_zero"]

def expected():
    return "\n".join([observed(0), observed(0)] +
                     [observed(bits & MAGNITUDE) for bits in VALUES for _ in OPERATIONS]) + "\n"

def traces(variant):
    if variant == "plain":
        return ""
    return {"dropped": "BAB", "duplicated": "AABAB",
            "ordinary_builtin": "AA", "ordinary_duplicated": "ABBABB"}.get(variant, "ABAB") * len(VALUES)
