use super::*;
use crate::tests::documentation_fixture as fixture;

#[test]
fn declaration_comment_spelling_is_exact_and_combines_attributes() {
    let docs = fixture::docs(
        4,
        JavaDocumentationStyle::Declaration,
        &["a\r\nb", "@x *\\ café"],
    );
    let output = modules(&docs);
    assert_eq!(
        output,
        "    /**\n     * a\n     * b\n     * &#64;x &#42;&#92; café\n     */\n"
    );
    assert_eq!(
        docs.iter().next().unwrap().1.presentation_len(),
        output.len()
    );
}

#[test]
fn presentation_reservations_match_structural_rendering_for_every_style() {
    for depth in [0, 1, 8, 128] {
        for style in [
            JavaDocumentationStyle::Declaration,
            JavaDocumentationStyle::ExplanatoryModule,
        ] {
            for attributes in [
                &[""][..],
                &["one", "two"],
                &["\r\n\r", "*/ \\u002a @tag", "日本語🦀\n"],
            ] {
                let docs = fixture::docs(depth, style, attributes);
                let output = modules(&docs);
                assert_eq!(
                    docs.iter().next().unwrap().1.presentation_len(),
                    output.len()
                );
                assert!(!output.contains(r"\u002a"));
            }
        }
    }
    let explanatory = fixture::docs(0, JavaDocumentationStyle::ExplanatoryModule, &["private"]);
    assert_eq!(
        modules(&explanatory),
        "/* Rust module 0000000000000007:0000000000000008\n * private\n */\n"
    );
    assert!(modules(&JavaDocumentation::default()).is_empty());
}
