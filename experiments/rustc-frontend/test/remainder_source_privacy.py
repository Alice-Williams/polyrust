"""External access must fail; deliberately exposing the helper must enable it."""
import shutil
import subprocess


def rejected(command, diagnostic):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=90)
    assert result.returncode != 0 and diagnostic in result.stderr, result.stderr


def java_privacy(run, work, runtime, classes, source, declaration):
    path = declaration["target"]["path"]
    name = path["member"]
    member = ".".join([path["package"], *path["owners"], name])
    consumer = work / "PrivateAccess.java"
    consumer.write_text(f"public final class PrivateAccess {{ public static double access() {{ return {member}(0.0); }} }}\n")
    flags = ["--release", "21", "-Xlint:all", "-Werror", "-implicit:none", "-sourcepath", ""]
    rejected([runtime / "javac", *flags, "-cp", classes, "-d", classes, consumer], "has private access")
    changed = work / "privacy-fault"
    changed.mkdir()
    exposed_classes = changed / "classes"
    shutil.copytree(classes, exposed_classes)
    exposed = changed / "Generated.java"
    text = source.read_text()
    needle = "private static double " + name + "("
    assert text.count(needle) == 1
    exposed.write_text(text.replace(needle, "public static double " + name + "("))
    run([runtime / "javac", *flags, "-cp", exposed_classes, "-d", exposed_classes, exposed, consumer])


def c_privacy(run, work, compiler, flags, source, leaf_object, symbol, label):
    changed = work / ("privacy-" + label)
    changed.mkdir()
    consumer = changed / "private.c"
    consumer.write_text(f"extern double {symbol}(double);\nint main(void) {{ return {symbol}(0.0) != 0.0; }}\n")
    client = changed / "private.o"
    run([compiler, *flags, "-c", consumer, "-o", client])
    rejected([compiler, client, leaf_object, "-o", changed / "forbidden"], symbol)
    text = source.read_text()
    needle = "static double " + symbol + "("
    assert text.count(needle) == 2
    exposed = changed / "exposed.c"
    exposed.write_text(text.replace(needle, "double " + symbol + "("))
    obj = changed / "exposed.o"
    run([compiler, *flags, "-c", exposed, "-o", obj])
    run([compiler, client, obj, "-o", changed / "exposed"])
    run([changed / "exposed"])
