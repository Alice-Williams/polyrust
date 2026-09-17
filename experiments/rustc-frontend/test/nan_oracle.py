"""Classification is computed with integer masks, not host floating operations."""
from binary64_oracle import VALUES

OPERATIONS = ["direct", "associated", "local", "imported", "forwarded", "shared", "record", "composed"]
LITERALS = ["positive_zero", "negative_zero"]


def classification(bits):
    return int((bits & 0x7ff0000000000000) == 0x7ff0000000000000 and
               (bits & 0x000fffffffffffff) != 0)


def expected():
    return "\n".join(["0", "0"] + [str(classification(bits)) for bits in VALUES
                                  for _ in OPERATIONS]) + "\n"


def traces(variant):
    if variant == "plain":
        return ""
    per_input = {"dropped": "N", "duplicated": "NNN"}.get(variant, "NN")
    return per_input * len(VALUES)
