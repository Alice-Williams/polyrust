"""Independent sign/category expectations using integers only."""
from binary64_oracle import VALUES, SIGN, MAGNITUDE, INFINITY, observed

OPERATIONS = {
    "direct": True, "nested": False, "local": True, "imported": True,
    "restored": False, "shared": True, "record": True, "composed": False,
}
LITERALS = {"negative_zero": SIGN, "restored_zero": 0}


def expected():
    lines = [observed(bits) for bits in LITERALS.values()]
    lines.extend(observed(bits ^ (SIGN if flip else 0))
                 for bits in VALUES for flip in OPERATIONS.values())
    return "\n".join(lines) + "\n"


def traces(variant):
    return {"plain": "", "traced": "GG", "dropped": "G", "duplicated": "GGG",
            "no_negation": "GG", "zero_minus": "GG"}[variant] * len(VALUES)
