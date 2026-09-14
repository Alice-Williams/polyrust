use super::JavaDocComment;
#[path = "doc_comments_native.rs"]
mod native;

#[test]
fn attribute_encoding_is_exact_and_not_idempotent_untrusted_input() {
    for (input, expected) in [
        ("ordinary café 日本語 🦀", "ordinary café 日本語 🦀"),
        ("a\r\nb\rc\nd", "a\nb\nc\nd"),
        ("/* */ **", "/&#42; &#42;/ &#42;&#42;"),
        (r"\u002a\u002f", "&#92;u002a&#92;u002f"),
        (r"\\u000a", "&#92;&#92;u000a"),
        ("<b>& @param", "&lt;b&gt;&amp; &#64;param"),
        ("&#92;u002a", "&amp;#92;u002a"),
        ("\0\t\u{1a}\u{85}", "&#x0;&#x9;&#x1A;&#x85;"),
        ("\u{2028}\u{202e}\u{2066}", "&#x2028;&#x202E;&#x2066;"),
        ("", ""),
    ] {
        let comment = JavaDocComment::new(input);
        assert_eq!(comment.text(), expected);
        assert_eq!(comment.encoded_len(), expected.len());
        assert_eq!(comment, JavaDocComment::new(input));
    }
}

pub(super) fn corpus() -> Vec<String> {
    let mut values = vec![
        "*/ static int injected; /*".into(),
        r"\u002a\u002f static int injected; \u002f\u002a".into(),
        r"\\u000a\u000d\uuuu002a".into(),
        "{@link java.lang.System} <script>alert(1)</script> &copy;".into(),
        "\r\n\r\n\ntrailing\r".into(),
    ];
    // Every valid Unicode scalar, including controls/noncharacters/supplementary
    // planes. The private payload invariant composes without lexer escape state.
    values.push((0..=0x10ffff).filter_map(char::from_u32).collect());
    let alphabet = ['*', '/', '\\', 'u', '0', 'a', '\r', '\n', '&', '@'];
    for first in alphabet {
        for second in alphabet {
            for third in alphabet {
                values.push([first, second, third].into_iter().collect());
            }
        }
    }
    values
}

#[test]
fn every_unicode_scalar_and_adversarial_triples_preserve_payload_invariants() {
    for input in corpus() {
        let comment = JavaDocComment::new(&input);
        assert!(!comment.text().contains(['\\', '*', '\r']));
        assert_eq!(
            comment.text().matches('\n').count(),
            input
                .replace("\r\n", "\n")
                .replace('\r', "\n")
                .matches('\n')
                .count()
        );
        assert_eq!(comment.encoded_len(), comment.text().len());
    }
}
