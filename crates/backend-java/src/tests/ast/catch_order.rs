use super::{
    DiagnosticCode, JavaBlock, JavaCatch, JavaDialect, JavaIdentifier, JavaKnownType,
    JavaPrimitive, JavaStmt, JavaType, admitted_throwable_is_supertype_of, fixture_declaration,
    structural_method, verify_fixture,
};

#[test]
fn catch_order_uses_the_admitted_throwable_hierarchy() {
    assert!(admitted_throwable_is_supertype_of(
        JavaKnownType::RuntimeException,
        JavaKnownType::IllegalArgumentException,
    ));
    assert!(admitted_throwable_is_supertype_of(
        JavaKnownType::RuntimeException,
        JavaKnownType::IllegalStateException,
    ));
    assert!(!admitted_throwable_is_supertype_of(
        JavaKnownType::IllegalArgumentException,
        JavaKnownType::RuntimeException,
    ));
    assert!(!admitted_throwable_is_supertype_of(
        JavaKnownType::RuntimeException,
        JavaKnownType::CharacterCodingException,
    ));

    let catch = |exception, binding: &str| JavaCatch {
        exception_type: JavaType::known(exception),
        binding: JavaIdentifier::from_portable(binding),
        body: JavaBlock::new(vec![]),
    };
    let method = |catches| {
        structural_method(
            "catchOrder",
            JavaType::primitive(JavaPrimitive::Void),
            vec![],
            JavaBlock::new(vec![JavaStmt::TryCatch {
                try_block: JavaBlock::new(vec![]),
                catches,
            }]),
        )
    };
    let invalid = fixture_declaration(vec![method(vec![
        catch(JavaKnownType::RuntimeException, "runtime"),
        catch(JavaKnownType::IllegalArgumentException, "argument"),
    ])]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], invalid)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("dominated by an earlier")
    }));

    let valid = fixture_declaration(vec![method(vec![
        catch(JavaKnownType::IllegalArgumentException, "argument"),
        catch(JavaKnownType::RuntimeException, "runtime"),
    ])]);
    let verification = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], valid)],
    );
    assert!(verification.is_ok(), "{verification:?}");
}
