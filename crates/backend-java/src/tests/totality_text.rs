//! Portable text must survive Java's modified-UTF-8 constant-pool boundary.

use super::totality_oracle::CompiledPackage;
use crate::JavaBackend;
use portable_build::{Text, portable_name, typed_list, typed_program};

#[test]
fn oversized_portable_text_compiles_and_preserves_contents() {
    for (case, value, expected) in [
        ("empty", String::new(), "\"\""),
        ("ascii-boundary", "a".repeat(65_534), "\"a\".repeat(65534)"),
        (
            "ascii-boundary-plus-one",
            "a".repeat(65_535),
            "\"a\".repeat(65535)",
        ),
        ("ascii", "a".repeat(65_536), "\"a\".repeat(65536)"),
        ("nul", "\0".repeat(32_768), "\"\\0\".repeat(32768)"),
        ("supplementary", "😀".repeat(10_923), "\"😀\".repeat(10923)"),
        (
            "mixed",
            "a\0ࠀ😀".repeat(30_000),
            "\"a\\0ࠀ😀\".repeat(30000)",
        ),
        ("megabyte", "x".repeat(1_048_576), "\"x\".repeat(1048576)"),
    ] {
        let program = typed_program(portable_name!("large_text"), |builder| {
            builder
                .function(
                    portable_name!("text"),
                    typed_list![],
                    Text::TYPE,
                    |body, _| body.text(value),
                )
                .builder
        });
        let manifest = JavaBackend.generate_typed(&program);
        let consumer = format!(
            r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {{
    private Consumer() {{}}
    public static void main(String[] args) {{
        if (!Generated.text().value().equals({expected})) throw new AssertionError();
    }}
}}
"#
        );
        CompiledPackage::new(&manifest, case).consumer(&consumer);
    }
}

#[test]
fn concatenation_does_not_fold_valid_chunks_into_an_oversized_constant() {
    let program = typed_program(portable_name!("large_concatenation"), |builder| {
        let builder = builder
            .function(
                portable_name!("direct"),
                typed_list![],
                Text::TYPE,
                |body, _| {
                    let left = body.text("a".repeat(60_000));
                    let right = body.text("b".repeat(60_000));
                    body.string_concat(left, right)
                },
            )
            .builder;
        builder
            .function(
                portable_name!("locals"),
                typed_list![],
                Text::TYPE,
                |body, _| {
                    let left = body.text("a".repeat(60_000));
                    body.let_value(portable_name!("left"), left, |body, left| {
                        let right = body.text("b".repeat(60_000));
                        body.let_value(portable_name!("right"), right, |body, right| {
                            let left = body.read_binding(left);
                            let right = body.read_binding(right);
                            body.string_concat(left, right)
                        })
                    })
                },
            )
            .builder
    });
    CompiledPackage::new(&JavaBackend.generate_typed(&program), "large-concatenation").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        String expected = "a".repeat(60000) + "b".repeat(60000);
        if (!Generated.direct().value().equals(expected)) throw new AssertionError();
        if (!Generated.locals().value().equals(expected)) throw new AssertionError();
    }
}
"#,
    );
}
