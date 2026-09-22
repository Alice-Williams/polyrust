"""Fail external access; deliberately exposing each private helper must pass."""
import shutil
import subprocess


def rejected(command, diagnostic):
    output = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=90)
    assert output.returncode != 0 and diagnostic in output.stderr, output.stderr


def java_privacy(run, work, runtime, classes, source, declaration):
    path = declaration["target"]["path"]
    member = ".".join([path["package"], *path["owners"], path["member"]])
    ty = "int" if declaration["result"] == "i32" else "long"
    directory = work / ("private-" + path["member"])
    directory.mkdir()
    client = directory / "PrivateAccess.java"
    client.write_text(f"public final class PrivateAccess {{ public static {ty} access() {{ return {member}(0); }} }}\n")
    flags = ["--release", "21", "-Xlint:all", "-Werror", "-implicit:none", "-sourcepath", ""]
    rejected([runtime / "javac", *flags, "-cp", classes, "-d", classes, client], "has private access")
    exposed_classes = directory / "classes"
    shutil.copytree(classes, exposed_classes)
    exposed = directory / "Generated.java"
    text = source.read_text()
    needle = f"private static {ty} {path['member']}("
    assert text.count(needle) == 1
    exposed.write_text(text.replace(needle, f"public static {ty} {path['member']}("))
    run([runtime / "javac", *flags, "-cp", exposed_classes, "-d", exposed_classes, exposed, client])


def c_privacy(run, work, compiler, flags, source, obj, declaration, width, label):
    symbol = declaration["symbol"]
    directory = work / ("private-" + symbol + label)
    directory.mkdir()
    client = directory / "private.c"
    client.write_text(f"#include <stdint.h>\nextern int{width}_t {symbol}(int{width}_t);\nint main(void) {{ return {symbol}(0) != 0; }}\n")
    client_obj = directory / "private.o"
    run([compiler, *flags, "-c", client, "-o", client_obj])
    rejected([compiler, client_obj, obj, "-o", directory / "forbidden"], symbol)
    text = source.read_text()
    needle = f"static int{width}_t {symbol}("
    assert text.count(needle) == 2
    exposed = directory / "exposed.c"
    exposed.write_text(text.replace(needle, f"int{width}_t {symbol}("))
    exposed_obj = directory / "exposed.o"
    run([compiler, *flags, "-c", exposed, "-o", exposed_obj])
    run([compiler, client_obj, exposed_obj, "-o", directory / "exposed"])
    run([directory / "exposed"])
