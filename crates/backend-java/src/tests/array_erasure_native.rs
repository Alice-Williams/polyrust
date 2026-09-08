//! Java casts change static types, not the identity or mutability of arrays.

#[test]
fn array_ownership_object_cast_native_witness() {
    use portable_build::{portable_name, typed_program};
    let program = typed_program(portable_name!("array_erasure"), |builder| builder);
    let manifest = crate::JavaBackend
        .generate_typed(&program)
        .expect("small package");
    let package = super::totality_oracle::CompiledPackage::new(&manifest, "array-erasure");
    package.consumer("package org.polyrust.consumer; public final class Consumer { private static final byte[] INTERNAL = new byte[1]; public static Object leak() { return (Object) INTERNAL; } public static void main(String[] args) { byte[] escaped = (byte[]) leak(); escaped[0] = 42; if (INTERNAL[0] != 42) throw new AssertionError(); } }");
}

#[test]
fn instanceof_patterns_native_controls() {
    use portable_build::{portable_name, typed_program};
    let program = typed_program(portable_name!("instanceof_patterns"), |builder| builder);
    let manifest = crate::JavaBackend
        .generate_typed(&program)
        .expect("small package");
    let package = super::totality_oracle::CompiledPackage::new(&manifest, "instanceof-patterns");
    package.consumer("package org.polyrust.consumer; public final class Consumer { private static final byte[] INTERNAL = new byte[1]; private static Object leak() { if (INTERNAL instanceof Object matched) return matched; throw new AssertionError(); } public static void main(String[] args) { String text = \"s\"; if (!(text instanceof Object) || !(new byte[1] instanceof Object)) throw new AssertionError(); if (!(text instanceof Object unconditional) || !unconditional.equals(text)) throw new AssertionError(); Object value = text; if (!(value instanceof String matched) || !matched.equals(text)) throw new AssertionError(); byte[] escaped = (byte[]) leak(); escaped[0] = 42; if (INTERNAL[0] != 42) throw new AssertionError(); } }");
}
