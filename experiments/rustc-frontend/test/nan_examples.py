"""Export actual unmodified compiler-generated artifacts, source and consumers."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy


def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "nan-classification"
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
    for filename in ["nan_leaf.rs", "nan_middle.rs", "nan_root.rs", "reference_nan.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        "# Checked Rust NaN classification\n\n"
        "Actual unmodified three-crate generated C and Java packages, handwritten clients "
        "and original Rust inputs. No generated static runtime or helper package.\n\n"
        "Proof target: //experiments/rustc-frontend:nan_native_test in the Linux dev container. "
        "Owners and consumers compile separately with GCC/Zig O0/O2 and Java21 strict lint. "
        "Exact NaN classification matches Rust and an independent integer-bit oracle. "
        "Test-only copies detect wrong equality, constant-false, dropped calls and duplicate calls.\n\n"
        "Scope: standard inherent f64::is_nan method and associated forms. Other arithmetic, "
        "float constants, casts and other inspection methods remain unsupported. "
        "NaN payload/sign preservation is not promised.\n")
