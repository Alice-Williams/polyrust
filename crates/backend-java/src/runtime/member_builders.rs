//! Typed Java runtime member construction.
use super::declaration_builders::identifier;

use super::expression_builders::{structural_field, this_value};
use super::statement_builders::illegal_state;
use crate::ast::{
    JavaBlock, JavaExpr, JavaField, JavaIdentifier, JavaMember, JavaMethod, JavaMethodDeclaration,
    JavaModifier, JavaParameter, JavaRuntimeMember, JavaStmt, JavaType,
};

pub(super) fn static_method(
    type_parameters: Vec<JavaIdentifier>,
    return_type: JavaType,
    name: &str,
    parameters: Vec<JavaParameter>,
    statements: Vec<JavaStmt>,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Static],
        type_parameters,
        return_type,
        name: identifier(name),
        parameters,
        body: Some(JavaBlock::new(statements)),
    })
}

pub(super) fn package_static_method(
    type_parameters: Vec<JavaIdentifier>,
    return_type: JavaType,
    name: &str,
    parameters: Vec<JavaParameter>,
    statements: Vec<JavaStmt>,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Static],
        type_parameters,
        return_type,
        name: identifier(name),
        parameters,
        body: Some(JavaBlock::new(statements)),
    })
}

pub(super) fn private_final_field(ty: JavaType, name: &str) -> JavaMember {
    JavaMember::Field(JavaField {
        declared: None,
        modifiers: vec![JavaModifier::Private, JavaModifier::Final],
        ty,
        name: identifier(name),
        initializer: None,
    })
}

pub(super) fn field_accessor(
    owner: JavaType,
    name: &str,
    ty: JavaType,
    member: JavaRuntimeMember,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: ty.clone(),
        name: identifier(member.name()),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
            structural_field(this_value(owner), name, ty),
        ))])),
    })
}
pub(super) fn guarded_accessor(
    owner: JavaType,
    name: &str,
    ty: JavaType,
    invalid: JavaExpr,
    message: &str,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: ty.clone(),
        name: identifier(name),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![
            JavaStmt::If {
                condition: invalid,
                then_block: JavaBlock::new(vec![illegal_state(message)]),
                else_block: None,
            },
            JavaStmt::Return(Some(structural_field(this_value(owner), name, ty))),
        ])),
    })
}
