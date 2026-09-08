//! Array allocation needs a reifiable component and source-ordered dimensions.

use super::{fixture_declaration, structural_method, verify_fixture};
use crate::ast::{
    JavaArrayOwnership, JavaBlock, JavaExpr, JavaExprKind, JavaIdentifier, JavaKnownType,
    JavaLiteral, JavaLocalFinality, JavaMember, JavaPrecedence, JavaPrimitive, JavaStmt, JavaType,
};
use crate::dialect::JavaDialect;

fn array(component: JavaType) -> JavaType {
    JavaType::Array {
        component: Box::new(component),
        ownership: JavaArrayOwnership::InternalMutable,
    }
}

fn method(index: usize, component: JavaType) -> JavaMember {
    let ty = array(component.clone());
    let mut member = structural_method(
        &format!("arrayCreation{index}"),
        JavaType::primitive(JavaPrimitive::Void),
        vec![],
        JavaBlock::new(vec![JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: ty.clone(),
            name: JavaIdentifier::new("values").unwrap(),
            value: Some(JavaExpr {
                ty,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::NewArray {
                    component,
                    length: Box::new(JavaExpr::literal(
                        JavaType::primitive(JavaPrimitive::Int),
                        JavaLiteral::I32(3),
                    )),
                },
            }),
        }]),
    );
    let JavaMember::Method(value) = &mut member else {
        unreachable!()
    };
    value
        .type_parameters
        .push(JavaIdentifier::new("T").unwrap());
    member
}

pub(super) fn legal_methods() -> Vec<JavaMember> {
    let int = JavaType::primitive(JavaPrimitive::Int);
    [
        int.clone(),
        JavaType::known(JavaKnownType::String),
        JavaType::generic(
            JavaKnownType::List,
            vec![JavaType::Wildcard { bound: None }],
        ),
        array(int.clone()),
        array(array(int)),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, component)| method(index, component))
    .collect()
}

#[test]
fn new_arrays_require_reifiable_components() {
    for member in legal_methods() {
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], fixture_declaration(vec![member]))]
            )
            .is_ok()
        );
    }
    let parameterized = JavaType::generic(
        JavaKnownType::List,
        vec![JavaType::known(JavaKnownType::String)],
    );
    let variable = JavaType::TypeVariable(JavaIdentifier::new("T").unwrap());
    for (index, component) in [
        parameterized.clone(),
        variable.clone(),
        array(parameterized),
        array(variable),
    ]
    .into_iter()
    .enumerate()
    {
        let result = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], fixture_declaration(vec![method(index, component)]))],
        );
        assert!(
            result.is_err(),
            "non-reifiable component {index} was admitted"
        );
    }
}
