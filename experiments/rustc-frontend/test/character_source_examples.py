"""Publish real generated artifacts from the native proof into test outputs."""
import os
from pathlib import Path
import shutil

from constant_export_scratch import writable_copy


def export(c, java, c_work, java_work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "character-values"
    destination.mkdir()
    writable_copy(c, destination / "c")
    writable_copy(java, destination / "java")
    clients = destination / "clients"
    clients.mkdir()
    shutil.copy2(c_work / "consumer.c", clients / "consumer.c")
    shutil.copy2(java_work / "Consumer.java", clients / "Consumer.java")
    original = destination / "rust-source"
    original.mkdir()
    fixtures = Path(__file__).resolve().parent.parent / "fixtures"
    for name in ["character_leaf.rs", "character_middle.rs", "character_root.rs", "reference_character_source.rs"]:
        shutil.copy2(fixtures / name, original / name)
    (destination / "README.md").write_text(
        "# Checked Rust character values\n\n"
        "Actual three-owner generated C/Java packages, original Rust fixtures and handwritten external clients. "
        "No custom runtime. Rust char remains a Unicode scalar; C stores uint32_t and Java stores int, not char.\n\n"
        "Run //experiments/rustc-frontend:character_source_native_test in the Linux Bazel environment. "
        "The proof compares 1,116,517 input rows and 19 literal boundaries against original Rust and an independent integer oracle. "
        "GCC14/Zig O0/O2 and GCC UBSan, Java21 normal/-Xint; strict separate compilation and external visibility probes.\n\n"
        "Consumers read little-endian (u32 left, u32 right, i32 marker) packets. They write 19 u32 literal values, "
        "then nine u32 observations per packet. Consumer protocol checks reject invalid scalar inputs; "
        "generated foreign APIs require their callers to satisfy the original Rust scalar domain, as described in source_types metadata.\n")
