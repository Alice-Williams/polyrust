"""Independent exact expectations for the original Rust arithmetic fixture."""
from arithmetic_oracle import MAGNITUDE, Operation as Op, rational, encode, result, observed
from arithmetic_cases import PAIRS

FMA_PAIR = (0x3ff0000000000000 + (1 << 25), 0x3ff0000000000000 - (1 << 26))
PAIRS = PAIRS + [FMA_PAIR]
OPERATIONS = ["add", "subtract", "multiply", "divide", "grouped", "separate", "nested_division"]
VARIANTS = ["plain", "traced", "operator", "swapped", "zero_sign", "grouping",
            "fused", "dropped", "duplicated", "reversed", "division_grouping"]


def values(left, right, variant):
    basic = [result(left, right, operation) for operation in Op]
    wanted = basic + [result(basic[0], right, Op.MULTIPLY),
                      result(basic[2], 0xbff0000000000000, Op.ADD),
                      result(left, result(right, left, Op.DIVIDE), Op.DIVIDE)]
    if variant == "operator":
        wanted[0] = result(left, right, Op.SUBTRACT)
    elif variant == "swapped":
        wanted[1] = result(right, left, Op.SUBTRACT)
    elif variant == "zero_sign" and wanted[0] & MAGNITUDE == 0:
        wanted[0] = 0
    elif variant == "grouping":
        wanted[4] = result(left, result(right, right, Op.MULTIPLY), Op.ADD)
    elif variant == "fused" and (left, right) == FMA_PAIR:
        wanted[5] = encode(rational(left) * rational(right) - 1)
    elif variant == "division_grouping":
        wanted[6] = result(result(left, right, Op.DIVIDE), left, Op.DIVIDE)
    return wanted


def expected(variant="plain", copies=2):
    lines = [observed(bits) for left, right in PAIRS for bits in values(left, right, variant) * copies]
    return "\n".join(lines) + "\n"


def traces(variant):
    if variant == "plain":
        return ""
    first = {"dropped": "B", "duplicated": "AAB", "reversed": "BA"}.get(variant, "AB")
    return ((first + "AB" * (len(OPERATIONS) - 1)) * 2) * len(PAIRS)


def inputs():
    return "".join(f"{left:016x} {right:016x}\n" for left, right in PAIRS)
