//! Catalogue identity includes the method name, not only compatible parameter types.
use crate::{
    ast::*,
    dialect::{JavaDialect, JavaKnownMethod},
    tests::{result_imports::fixture, source_dependency_fixture as source},
};
use portable_codegen::verify_unresolved_package;

#[test]
fn known_member_spelling_cannot_replace_original_method_identity() {
    let expression = |name: &str| JavaExpr {
        ty: source::int(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: JavaType::known(JavaKnownType::String),
                name: source::name(name),
                signature: Box::new(JavaKnownMethod::StringLength.signature()),
                origin: JavaMemberOrigin::Known(JavaKnownMethod::StringLength),
            },
            receiver: Some(Box::new(JavaExpr::literal(
                JavaType::known(JavaKnownType::String),
                JavaLiteral::String("a".into()),
            ))),
            arguments: vec![],
        },
    };
    let valid = source::certify(fixture::consumer(
        Default::default(),
        source::int(),
        vec![],
        expression("length"),
    ));
    assert!(fixture::text(&valid).contains(".length()"));
    // hashCode exists with the same invocation shape, but would change semantics.
    // nonexistent would instead produce syntactically parsed but uncompilable Java.
    for name in ["hashCode", "nonexistent"] {
        let draft = fixture::consumer(Default::default(), source::int(), vec![], expression(name));
        let errors = verify_unresolved_package(&JavaDialect, draft).unwrap_err();
        assert!(
            errors.iter().any(|error| error.code
                == portable_diagnostics::DiagnosticCode::UnresolvedReference
                || error.code == portable_diagnostics::DiagnosticCode::InvalidInvocation),
            "{name}: {errors:?}"
        );
    }
}
