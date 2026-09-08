//! Independent encoding boundaries and rejection before AST certification.

use super::{
    DiagnosticCode, JavaBlock, JavaDialect, JavaExpr, JavaKnownType, JavaLiteral, JavaStmt,
    JavaType, fixture_declaration, structural_method, verify_fixture,
};
use portable_codegen::TargetAstBuilder;

#[test]
fn literal_encoded_boundaries_are_enforced_before_certification() {
    for (value, accepted) in [
        ("a".repeat(65_534), true),
        ("a".repeat(65_535), false),
        ("\0".repeat(32_767), true),
        ("\0".repeat(32_768), false),
        ("😀".repeat(10_922), true),
        ("😀".repeat(10_923), false),
    ] {
        let string = JavaType::known(JavaKnownType::String);
        let method = structural_method(
            "text",
            string.clone(),
            vec![],
            JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                string,
                JavaLiteral::String(value),
            )))]),
        );
        let result = verify_fixture(
            TargetAstBuilder::new(JavaDialect),
            vec![(vec![], fixture_declaration(vec![method]))],
        );
        if accepted {
            assert!(result.is_ok(), "{result:?}");
        } else {
            assert!(
                result
                    .unwrap_err()
                    .iter()
                    .any(|error| error.code == DiagnosticCode::InvalidStructure
                        && error.message.contains("modified-UTF-8 constant limit"))
            );
        }
    }
}

#[test]
fn exact_utf16_literal_accounting_includes_nul_and_surrogates() {
    for (unit, count, accepted) in [
        (0, 32_767, true),
        (0, 32_768, false),
        (0x7f, 65_534, true),
        (0x7f, 65_535, false),
        (0x800, 21_844, true),
        (0x800, 21_845, false),
        (0xd800, 21_844, true),
        (0xd800, 21_845, false),
    ] {
        let errors = crate::ast::literal_limits::verify_literal(
            &JavaLiteral::Utf16Units(vec![unit; count]),
            &JavaType::known(JavaKnownType::String),
        );
        assert_eq!(
            errors.is_empty(),
            accepted,
            "{unit:x} x {count}: {errors:?}"
        );
    }
}

#[test]
fn chunks_preserve_scalar_boundaries_and_modified_utf8_bytes() {
    for value in [
        String::new(),
        "a".repeat(65_535),
        "\0".repeat(100_000),
        format!("{}😀\0ࠀz", "x".repeat(65_533)),
        "a\0ࠀ😀".repeat(30_000),
    ] {
        let chunks = crate::ast::literal_limits::string_chunks(&value);
        assert!(!chunks.is_empty());
        assert_eq!(chunks.concat(), value);
        for chunk in chunks {
            // Independent scalar-based formula, not production's UTF-16-unit sum.
            let bytes: usize = chunk
                .chars()
                .map(|scalar| match scalar as u32 {
                    0 => 2,
                    1..=0x7f => 1,
                    0x80..=0x7ff => 2,
                    0x800..=0xffff => 3,
                    _ => 6,
                })
                .sum();
            assert!(bytes <= 65_534, "{bytes}");
        }
    }
}
