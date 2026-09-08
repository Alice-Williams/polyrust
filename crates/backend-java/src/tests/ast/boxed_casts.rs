//! Independent complete matrix for the admitted boxed-to-primitive cast grammar.
use super::{
    DiagnosticCode, JavaBlock, JavaDialect, JavaExpr, JavaExprKind, JavaIdentifier, JavaMember,
    JavaPrecedence, JavaPrimitive, JavaStmt, JavaType, fixture_declaration, parameter,
    structural_method, verify_fixture,
};
use portable_codegen::TargetAstBuilder;

const PRIMITIVES: [JavaPrimitive; 6] = [
    JavaPrimitive::Byte,
    JavaPrimitive::Char,
    JavaPrimitive::Int,
    JavaPrimitive::Long,
    JavaPrimitive::Double,
    JavaPrimitive::Boolean,
];
// Columns and rows follow PRIMITIVES. Deliberately independent of production.
const LEGAL: [[bool; 6]; 6] = [
    [true, false, true, true, true, false],
    [false, true, true, true, true, false],
    [false, false, true, true, true, false],
    [false, false, false, true, true, false],
    [false, false, false, false, true, false],
    [false, false, false, false, false, true],
];

fn cast_method(source: JavaPrimitive, target: JavaPrimitive) -> JavaMember {
    let result = JavaType::primitive(target);
    structural_method(
        &format!("cast_{source:?}_to_{target:?}"),
        result.clone(),
        vec![parameter(JavaType::Boxed(source), "value")],
        JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: result.clone(),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Cast {
                target: result,
                value: Box::new(JavaExpr::local(
                    JavaType::Boxed(source),
                    JavaIdentifier::from_portable("value"),
                )),
            },
        }))]),
    )
}

#[test]
fn boxed_casts_allow_only_unboxing_and_widening() {
    for (row, source) in PRIMITIVES.into_iter().enumerate() {
        for (column, target) in PRIMITIVES.into_iter().enumerate() {
            let verified = verify_fixture(
                TargetAstBuilder::new(JavaDialect),
                vec![(
                    vec![],
                    fixture_declaration(vec![cast_method(source, target)]),
                )],
            );
            if LEGAL[row][column] {
                assert!(verified.is_ok(), "{source:?} -> {target:?}: {verified:?}");
            } else {
                let errors = verified.expect_err(&format!("{source:?} -> {target:?}"));
                assert!(
                    errors
                        .iter()
                        .any(|error| error.code == DiagnosticCode::TypeMismatch
                            && error.message.contains("cast is not legal")),
                    "{errors:?}"
                );
            }
        }
    }
}

/// The real certified renderer/compiler oracle includes every admitted pair.
pub(super) fn legal_methods() -> Vec<JavaMember> {
    let mut methods = Vec::new();
    for (row, source) in PRIMITIVES.into_iter().enumerate() {
        for (column, target) in PRIMITIVES.into_iter().enumerate() {
            if LEGAL[row][column] {
                methods.push(cast_method(source, target));
            }
        }
    }
    assert_eq!(methods.len(), 15);
    methods
}
