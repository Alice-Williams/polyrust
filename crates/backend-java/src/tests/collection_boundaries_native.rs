//! Native witnesses that exposed mutable builders retain caller-visible aliases.

#[test]
fn mutable_collection_native_aliasing_witness() {
    use portable_build::{portable_name, typed_program};
    let program = typed_program(portable_name!("collection_boundaries"), |builder| builder);
    let manifest = crate::JavaBackend
        .generate_typed(&program)
        .expect("small package");
    let package = super::totality_oracle::CompiledPackage::new(&manifest, "collection-boundaries");
    package.consumer(
        r#"
package org.polyrust.consumer;
public final class Consumer {
    private static final java.util.ArrayList<Integer> INTERNAL = new java.util.ArrayList<>();
    public static Object leak() { return (Object) INTERNAL; }
    public static java.util.LinkedHashMap<String, Integer> identity(
            java.util.LinkedHashMap<String, Integer> value) { return value; }
    public static void main(String[] args) {
        INTERNAL.add(1);
        ((java.util.ArrayList<?>) leak()).clear();
        if (!INTERNAL.isEmpty()) throw new AssertionError();
        var source = new java.util.LinkedHashMap<String, Integer>();
        source.put("key", 1);
        identity(source).clear();
        if (!source.isEmpty()) throw new AssertionError();
        INTERNAL.add(1);
        if (INTERNAL instanceof Object matched) ((java.util.ArrayList<?>) matched).clear();
        if (!INTERNAL.isEmpty()) throw new AssertionError();
    }
}
"#,
    );
}
