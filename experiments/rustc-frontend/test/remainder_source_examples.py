"""Export generated packages verbatim, with original input and baseline clients."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy


def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "floating-remainder"
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
    for filename in ["remainder_leaf.rs", "remainder_middle.rs", "remainder_root.rs", "reference_remainder_source.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        "# Checked Rust binary64 remainder\n\n"
        "Actual unmodified three-crate C and Java packages, original Rust and handwritten clients. "
        "No generated runtime or helper package. C consumers link the certified standard math library (-lm).\n\n"
        "Proof: //experiments/rustc-frontend:remainder_native_test. "
        "Three separately compiled owners and consumer; GCC14/Zig O0/O2, Java21 strict lint. "
        "22,832 exact target bit/category observations per run agree with 11,416 native Rust cases and an independent "
        "integer/rational binary64 oracle. Instrumented native Rust operand traces also agree. "
        "Six compiling value/trace faults are detected; external private access fails, "
        "with exposing-private controls proving those checks.\n\n"
        "Scope: built-in f64 % with truncating quotient and ordered once-only operands. "
        "No IEEE nearest-quotient or Euclidean remainder substitution is permitted. NaN payload/sign is not promised.\n")
