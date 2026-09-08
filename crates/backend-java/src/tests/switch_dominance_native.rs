//! Pinned compiler controls for the known JDK type-pattern dominance relation.

#[test]
fn java21_rejects_dominated_known_subtype_patterns() {
    use portable_build::{portable_name, typed_program};
    let program = typed_program(portable_name!("switch_dominance"), |builder| builder);
    let manifest = crate::JavaBackend
        .generate_typed(&program)
        .expect("small package");
    let package = super::totality_oracle::CompiledPackage::new(&manifest, "switch-dominance");
    for (parent, child) in [
        ("java.util.List<?>", "java.util.ArrayList<?>"),
        ("java.util.Map<?, ?>", "java.util.LinkedHashMap<?, ?>"),
        ("RuntimeException", "IllegalArgumentException"),
        ("RuntimeException", "IllegalStateException"),
    ] {
        package.rejects_generated_member(&format!("public static void dominated(Object value) {{ switch (value) {{ case {parent} first -> {{}} case {child} second -> {{}} default -> {{}} }} }}"), "dominated");
    }
}
