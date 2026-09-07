//! Java lowering: declaration builders.

use super::ExprPlan;
use crate::ast::{
    JavaBlock, JavaConstructor, JavaIdentifier, JavaMember, JavaMethod, JavaMethodDeclaration,
    JavaModifier, JavaParameter, JavaStmt, JavaType,
};
use portable_ir::v0::Visibility;

pub(crate) fn identifier(value: &str) -> JavaIdentifier {
    JavaIdentifier::from_portable(value)
}

pub(crate) fn visibility_modifier(value: Visibility) -> JavaModifier {
    match value {
        Visibility::Public => JavaModifier::Public,
        Visibility::Package => JavaModifier::Private,
    }
}

pub(crate) fn private_constructor(name: &str) -> JavaConstructor {
    JavaConstructor {
        modifiers: vec![JavaModifier::Private],
        name: identifier(name),
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    }
}

pub(super) fn public_factory_method(
    name: &str,
    return_type: JavaType,
    parameters: Vec<JavaParameter>,
    mut result: ExprPlan,
) -> JavaMember {
    result.statements.push(JavaStmt::Return(Some(result.value)));
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters: vec![],
        return_type,
        name: JavaIdentifier::new(name).expect("internal Java factory identifier is valid"),
        parameters,
        body: Some(JavaBlock::new(result.statements)),
    })
}

pub(super) fn length_prefixed_type(category: &str, payload: &str) -> String {
    format!("{category}{}_{}", payload.len(), payload)
}
