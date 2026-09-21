"""Independent integer/rational expectations and original operand-call traces."""
from arithmetic_oracle import observed
from remainder_cases import PAIRS
from remainder_oracle import result
from remainder_faults import changed

VARIANTS = ["plain", "traced", "nearest", "swapped", "zero_sign", "dropped", "duplicated", "reversed"]


def expected(variant="plain", copies=2):
    operation = (lambda a, b: changed(a, b, variant)) if variant in ("nearest", "swapped", "zero_sign") else result
    return "\n".join(observed(operation(a, b)) for a, b in PAIRS for _ in range(copies)) + "\n"


def inputs():
    return "".join(f"{a:016x} {b:016x}\n" for a, b in PAIRS)


def traces(variant):
    if variant == "plain":
        return ""
    return {"dropped": "B", "duplicated": "AAB", "reversed": "BA"}.get(variant, "AB") * 2 * len(PAIRS)
