"""Exact independently specified truth, including shadowed constant values."""
CASES = [
    ("simple", "i32", 4, 4),
    ("min32", "i32", -2147483648, -2147483648),
    ("max32", "i32", 2147483647, 2147483647),
    ("min64", "i64", -9223372036854775808, -9223372036854775808),
    ("max64", "i64", 9223372036854775807, 9223372036854775807),
    ("computed", "i32", 62, 62),
    ("bool_true", "bool", 1, 1),
    ("bool_false", "bool", 0, 0),
    ("forward", "i64", 9007199254740993, 9007199254740993),
    ("unused", "i32", 9, 9),
    ("shadow", "i32", 43, 41),
    ("nested", "i32", 17, 17),
    ("branch", "i64", 9223372036854775807, -9223372036854775808),
    ("record", "i64", -9007199254740993, 9007199254740993),
    ("imported", "i64", -9007199254740993, 9007199254740993),
]

def expected(mutated=False):
    return "".join(str(17 if mutated and name == "computed" else case[flag + 2]) + "\n"
                   for flag in [0, 1] for case in CASES for name in [case[0]])
