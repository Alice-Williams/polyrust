"""Export actual unmodified compiler-generated artifacts, source and consumers."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy


def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "binary64-values"
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
    for filename in ["binary64_leaf.rs", "binary64_middle.rs", "binary64_root.rs", "reference_binary64.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        "# Checked Rust binary64 values\n\n"
        "Actual unmodified three-crate generated C and Java packages, handwritten clients "
        "and original Rust inputs. No generated static runtime or helper package.\n\n"
        "Proof target: //experiments/rustc-frontend:binary64_native_test in the Linux dev container. "
        "Owners and consumers compile separately with GCC/Zig O0/O2 and Java21 strict lint. "
        "Exact finite bits, signed zero, nonfinite classification and six comparisons match "
        "Rust and an independent integer-bit oracle. Test-only trace copies detect reordered "
        "and dropped calls; literal mutants detect signed-zero/subnormal corruption.\n\n"
        "Scope: finite literals and f64 transport/comparisons. Floating arithmetic/constants, "
        "casts and methods remain unsupported. NaN payload/sign preservation is not promised.\n")
