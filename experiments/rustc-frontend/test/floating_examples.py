"""Export actual unmodified compiler-generated artifacts, source and consumers."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy


def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "floating-negation"
    destination.mkdir()
    writable_copy(java, destination / "java")
    writable_copy(c, destination / "c")
    clients = destination / "clients"
    clients.mkdir()
    for parent, filename in [("java-plain", "Consumer.java"), ("c-plain", "consumer.c")]:
        shutil.copy2(work / parent / filename, clients / filename)
    sources = destination / "rust-source"
    sources.mkdir()
    fixtures = Path(__file__).resolve().parent.parent / "fixtures"
    for filename in ["floating_leaf.rs", "floating_middle.rs", "floating_root.rs", "reference_floating.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        "# Checked Rust floating negation\n\n"
        "Actual unmodified three-crate generated C and Java packages, handwritten clients "
        "and original Rust inputs. No generated static runtime or helper package.\n\n"
        "Proof target: //experiments/rustc-frontend:floating_native_test in the Linux dev container. "
        "Owners and consumers compile separately with GCC/Zig O0/O2 and Java21 strict lint. "
        "Exact non-NaN sign bits and NaN classification match Rust and an independent integer-bit oracle. "
        "Test-only copies detect zero-minus, omitted negation, dropped calls and duplicate calls.\n\n"
        "Scope: built-in f64 unary minus, retaining finite negative literals. Other arithmetic, "
        "float constants, casts, methods and overloaded negation remain unsupported. "
        "NaN payload/sign preservation is not promised.\n")
