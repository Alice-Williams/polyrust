"""Independent infinity bits; actual separately recompiled Java21 consumers."""
from pathlib import Path
import re
import subprocess
import sys

sys.path.insert(0, str(Path(sys.argv[-1]).resolve()))
from infinite_constant_oracle import INFINITY, SIGN, faulty

VALUES = [INFINITY, SIGN | INFINITY]
PREFIX = "org.polyrust.generated."
OWNER = PREFIX + "r000000000000035c.Generated"
FACADE = PREFIX + "r0000000000000925.Generated"
READER = PREFIX + "r0000000000000926.Generated"


def run(command):
    output = subprocess.run([str(arg) for arg in command], capture_output=True,
                            text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    assert output.stderr == "", (command, output.stderr)
    return output.stdout


def source(root, owner):
    return root / "src/main/java" / (owner.replace(".", "/") + ".java")


def client():
    expressions = ([f"{OWNER}.constant{i}" for i in range(2)]
                   + [f"{OWNER}.read{i}()" for i in range(2)]
                   + [f"{READER}.read{i}()" for i in range(2)])
    statements = "\n".join('System.out.printf("%016x%n", Double.doubleToRawLongBits(' + value + '));'
                           for value in expressions)
    reflection = f"""
        Class<?> facade = Class.forName("{FACADE}");
        if (facade.getDeclaredFields().length != 0 || facade.getDeclaredMethods().length != 0
            || facade.getDeclaredConstructors().length != 1
            || !java.lang.reflect.Modifier.isPrivate(facade.getDeclaredConstructors()[0].getModifiers()))
            throw new AssertionError("alias storage or wrappers");
    """
    return ("public final class Consumer { public static void main(String[] args) throws Exception {\n"
            + statements + reflection + "\n} }\n")


def mutate(root, variant):
    path = source(root, OWNER)
    original = path.read_text()
    replacements = {
        "sign_loss": ["Double.POSITIVE_INFINITY"] * 2,
        "finite_clamp": ["0x1.fffffffffffffp1023", "-0x1.fffffffffffffp1023"],
        "zero": ["0.0", "0.0"],
    }
    changed, count = re.subn(r"(\bconstant(\d+)\s*=\s*)[^;]+;",
                            lambda match: match[1] + replacements[variant][int(match[2])] + ";",
                            original)
    assert count == 2 and changed != original
    path.write_text(changed)


def observe(root, javac, java):
    classes = root / "classes"
    classes.mkdir()
    flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
             "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
    for owner in [OWNER, FACADE, READER]:
        run([javac, *flags, source(root, owner)])
    driver = root / "Consumer.java"
    driver.write_text(client())
    run([javac, *flags, driver])
    return [run([java, *options, "-cp", classes, "Consumer"]) for options in [[], ["-Xint"]]]


def expected(values):
    return "".join(f"{value:016x}\n" for value in values) * 3


def main():
    root, javac, java = [Path(arg).resolve() for arg in sys.argv[1:4]]
    assert run([javac, "-version"]).startswith("javac 21.")
    original = {path.relative_to(root): path.read_bytes() for path in root.rglob("*.java")}
    assert len(original) == 3
    assert observe(root, javac, java) == [expected(VALUES)] * 2
    for variant in ["sign_loss", "finite_clamp", "zero"]:
        directory = root.parent / variant
        directory.mkdir()
        for path, contents in original.items():
            target = directory / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(contents)
        mutate(directory, variant)
        observed = observe(directory, javac, java)
        truth = expected([faulty(value, variant) for value in VALUES])
        assert truth != expected(VALUES)
        assert observed == [truth] * 2
        assert all(value != expected(VALUES) for value in observed)
    assert all((root / path).read_bytes() == contents for path, contents in original.items())
    print("Both signs: owned fields/readers, aliased imported readers; separate Java21 compilation; "
          "normal/-Xint; three compiling faults detected after all dependents recompiled")


if __name__ == "__main__":
    main()
