"""Independent scalar integers, source kinds and named read order."""
CONSTANTS = {
    "ZERO": 0, "ASCII": 65, "SAME_INTEGER": 65,
    "BEFORE_SURROGATES": 0xd7ff, "AFTER_SURROGATES": 0xe000,
    "NONCHARACTER": 0xffff, "SUPPLEMENTARY": 0x10000,
    "MAXIMUM": 0x10ffff, "COMPUTED": 0x1f980, "COPIED": 0x10ffff,
}
READS = dict((name.lower(), value) for name, value in CONSTANTS.items())
READS.update(other_same=65, other_maximum=0x10ffff, alias=0x10ffff,
             own=0x1f980, private=0xe000, local=0x10000, inherent=0xffff, comparison=1)


def text(values):
    return "".join(str(value) + "\n" for value in values)


def reject(check):
    try:
        check()
    except AssertionError:
        return
    raise AssertionError("corrupted evidence accepted")
