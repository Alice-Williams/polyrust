"""Certified Java Int storage, full scalar transport and real wrong-representation faults."""
from pathlib import Path
import re
import struct
import subprocess
import sys

ROOT, JAVAC, JAVA, ORACLE = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(ORACLE))
from character_oracle import BOUNDARIES, PAIRS, MAXIMUM, SCALAR_COUNT, scalar, flags, utf16

INPUT = struct.Struct("<II")
OUTPUT = struct.Struct("<IIIIIII")
OWNER = [f"org.polyrust.generated.r{crate:016x}.Generated" for crate in (941, 942, 943)]
CLIENT = r"""public final class Consumer {
private Consumer() {}
private static int read32(byte[] bytes, int start) {
    return (bytes[start] & 255) | ((bytes[start + 1] & 255) << 8)
        | ((bytes[start + 2] & 255) << 16) | ((bytes[start + 3] & 255) << 24);
}
public static void main(String[] args) throws java.io.IOException {
LITERALS
    java.io.BufferedInputStream input = new java.io.BufferedInputStream(System.in);
    java.io.DataOutputStream output = new java.io.DataOutputStream(
        new java.io.BufferedOutputStream(System.out));
    byte[] bytes = new byte[8];
    for (;;) {
        int count = input.readNBytes(bytes, 0, bytes.length);
        if (count == 0) break;
        if (count != bytes.length) throw new java.io.IOException("truncated packet");
        int left = read32(bytes, 0), right = read32(bytes, 4);
        int[] results = {
            FIRST.identity(left), FIRST.local(left), SECOND.forward(left), THIRD.forward(left),
            FIRST.select(true, left, right), FIRST.select(false, left, right),
            COMPARISONS
        };
        for (int value : results) output.writeInt(Integer.reverseBytes(value));
    }
    output.flush();
}
}
"""
CLIENT = CLIENT.replace("LITERALS", "\n".join(
    f"    if (FIRST.literal{i}() != {value}) throw new AssertionError(\"literal{i}\");"
    for i, value in enumerate(BOUNDARIES)))
CLIENT = CLIENT.replace("COMPARISONS", " | ".join(
    f"((FIRST.compare{i}(left, right) ? 1 : 0) << {i})" for i in range(6)))
for token, owner in zip(["FIRST", "SECOND", "THIRD"], OWNER):
    CLIENT = CLIENT.replace(token, owner)


def run(command, data=None, success=True):
    output = subprocess.run([str(arg) for arg in command], input=data,
                            capture_output=True, timeout=180)
    assert (output.returncode == 0) == success, (command, output.stdout, output.stderr)
    return output


def compile_source(source, classes, success=True):
    return run([JAVAC, "--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes, source],
               success=success)


def rows(full):
    if full:
        for value in range(MAXIMUM + 1):
            if scalar(value):
                yield value, 0
    else:
        yield from ((value, 0) for value in BOUNDARIES)
    yield from PAIRS


def expected(left, right, variant):
    transported = left
    if variant == "byte":
        transported &= 255
    elif variant == "utf16":
        transported &= 65535
    comparison = flags(left, right)
    if variant == "reverse":
        comparison = flags(right, left)
    elif variant == "utf16_order":
        comparison = flags(utf16(left), utf16(right))
    return transported, left, transported, transported, left, right, comparison


def mutate(text, variant):
    if variant in ("byte", "utf16"):
        pattern = r"(\bint identity\(final int input\)\s*\{\s*return\s+)([^;]+)(;)"
        def narrowing(match):
            assert match[2] == "input"
            value = "(input & 255)" if variant == "byte" else "((char) input)"
            return match[1] + value + match[3]
        changed, count = re.subn(pattern, narrowing, text)
        assert count == 1 and changed != text
        return changed
    assert variant in ("reverse", "utf16_order")
    changed = text
    for index, op in enumerate(["==", "!=", "<", "<=", ">", ">="]):
        pattern = rf"(\bboolean compare{index}\(final int left, final int right\)\s*\{{\s*return\s+)([^;]+)(;)"
        def comparison(match):
            assert "left" in match[2] and "right" in match[2]
            if variant == "reverse":
                value = f"(right {op} left)"
            else:
                value = ("new String(Character.toChars(left)).compareTo("
                         f"new String(Character.toChars(right))) {op} 0")
            return match[1] + value + match[3]
        changed, count = re.subn(pattern, comparison, changed)
        assert count == 1
    assert changed != text
    return changed


def main():
    assert run([JAVAC, "-version"]).stdout.startswith(b"javac 21.")
    originals = []
    for index in range(3):
        sources = list((ROOT / f"owner{index}").rglob("*.java"))
        assert len(sources) == 1
        originals.append((sources[0], sources[0].read_bytes()))
    for variant in ["valid", "byte", "utf16", "reverse", "utf16_order"]:
        cases = list(rows(variant == "valid"))
        assert len(cases) == (SCALAR_COUNT if variant == "valid" else len(BOUNDARIES)) + len(PAIRS)
        data = b"".join(INPUT.pack(*row) for row in cases)
        truth = b"".join(OUTPUT.pack(*expected(*row, "valid")) for row in cases)
        wanted = truth if variant == "valid" else b"".join(OUTPUT.pack(*expected(*row, variant)) for row in cases)
        assert (wanted == truth) == (variant == "valid")
        directory, classes = ROOT / variant, ROOT / variant / "classes"
        classes.mkdir(parents=True)
        for index, (original, content) in enumerate(originals):
            text = content.decode()
            assert all(fragment not in text for fragment in ["Runtime", "import ", "Character", "String", "char "])
            if index == 0 and variant != "valid":
                text = mutate(text, variant)
            source = directory / f"owner{index}" / original.relative_to(ROOT / f"owner{index}")
            source.parent.mkdir(parents=True)
            source.write_text(text)
            compile_source(source, classes)
        client = directory / "Consumer.java"
        client.write_text(CLIENT)
        compile_source(client, classes)
        for options in [[], ["-Xint"]]:
            output = run([JAVA, *options, "-cp", classes, "Consumer"], data)
            assert output.stdout == wanted, (variant, options, "exact scalar transport/order")
            assert not output.stderr
            malformed = run([JAVA, *options, "-cp", classes, "Consumer"], b"x", success=False)
            assert b"truncated packet" in malformed.stderr
    # Missing original owner and deliberately malformed syntax must not compile.
    negative = ROOT / "negative"
    negative.mkdir()
    classes = negative / "classes"
    classes.mkdir()
    source, _ = originals[1]
    assert compile_source(source, classes, success=False).stderr
    original, content = originals[0]
    broken, count = re.subn(r"(\breturn\b[^;]+);", r"\1", content.decode(), count=1)
    assert count == 1
    source = negative / original.name
    source.write_text(broken)
    assert compile_source(source, classes, success=False).stderr
    assert all(path.read_bytes() == content for path, content in originals)
    print(f"{SCALAR_COUNT} scalars + {len(PAIRS)} pairs; 19 literals; three original owners; "
          "strict separate Java21 compilation, normal/-Xint; four compiling narrowing/order "
          "faults detected; malformed-input and compile-negative controls pass")


if __name__ == "__main__":
    main()
