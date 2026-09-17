"""Export generated packages verbatim, with original input and baseline clients."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy


def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "floating-arithmetic"
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
    for filename in ["arithmetic_leaf.rs", "arithmetic_middle.rs", "arithmetic_root.rs", "reference_arithmetic_source.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        "# Checked Rust binary64 arithmetic\n\n"
        "Actual unmodified three-crate C and Java packages, original Rust and handwritten clients. "
        "No generated runtime, helper package or math-library dependency.\n\n"
        "Proof: //experiments/rustc-frontend:arithmetic_native_test. "
        "Three separately compiled owners and consumer; GCC14/Zig O0/O2, Java21 strict lint. "
        "45,038 exact bit/category observations per run agree with native Rust and an independent "
        "integer/rational binary64 oracle. Nine compiling value/trace faults are detected.\n\n"
        "Scope: built-in f64 +, -, *, / with ordered once-only operands and preserved grouping. "
        "No reassociation or fused operation is permitted. NaN payload/sign is not promised.\n")
