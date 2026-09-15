"""Independent exact-integer expectations: never derive truth from generated code."""
CASES = [
    ("min32", "i32", -2147483648, -2147483648),
    ("max32", "i32", 2147483647, 2147483647),
    ("min64", "i64", -9223372036854775808, -9223372036854775808),
    ("max64", "i64", 9223372036854775807, 9223372036854775807),
    ("exact", "i64", 9007199254740993, 9007199254740993),
    ("negative", "i64", -9007199254740993, -9007199254740993),
    ("computed", "i32", 62, 62),
    ("primitive32", "i32", -2147483648, -2147483648),
    ("primitive64", "i64", 9223372036854775807, 9223372036854775807),
    ("associated", "i64", 9223372036854775000, 9223372036854775000),
    ("left", "i32", 41, 41),
    ("right", "i32", 43, 43),
    ("alias", "i32", 41, 41),
    ("bool_true", "bool", 1, 1),
    ("bool_false", "bool", 0, 0),
    ("branch", "i64", 9223372036854775807, -9223372036854775808),
    ("record", "i64", -9007199254740993, 9007199254740993),
    ("imported", "i64", -9007199254740993, 9007199254740993),
    ("shared_local", "i64", 9007199254740993, 9007199254740993),
]


def expected(mutated=False):
    return "".join(str(17 if mutated and name == "computed" else case[flag + 2]) + "\n"
                   for flag in [0, 1] for case in CASES for name in [case[0]])
