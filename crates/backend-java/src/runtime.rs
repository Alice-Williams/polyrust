use crate::ast::{
    JavaBlock, JavaConstructor, JavaDeclarationKind, JavaFileItem, JavaHeritage, JavaMember,
    JavaModifier, JavaPrimitive, JavaStmt, JavaType, JavaTypeDeclaration, JavaVisibility,
};
use crate::dialect::JavaRuntimeHelper;
use bytes_storage::bytes_members;
use core_types::core_members;
use declaration_builders::identifier;
use dispatch::runtime_methods;
use expression_builders::bool_literal;
use member_builders::static_method;
use tagged_values::tagged_members;
mod core_types;

mod equality;

mod tagged_values;

mod option;

mod value_result;

mod bytes_storage;

mod dispatch;

mod string_replace;

mod string_transform;

mod unicode;

mod unicode_validation;

mod checked_integer;

mod float;

mod immutable_lists;

mod statement_builders;

mod bytes_operations;

mod declaration_builders;
mod member_builders;

mod expression_builders;

mod call_builders;

pub(crate) fn shell_item() -> JavaFileItem {
    JavaFileItem::Type {
        conformances: crate::ast::JavaConformanceInventory::structural().into(),
        declared: vec![],
        declaration: JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            modifiers: vec![],
            name: identifier("Runtime"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Private],
                name: identifier("Runtime"),
                parameters: vec![],
                body: JavaBlock::new(vec![]),
            })],
        },
    }
}

pub fn helper_items(helper: JavaRuntimeHelper) -> Vec<JavaFileItem> {
    let members = match helper {
        JavaRuntimeHelper::Core => core_members(),
        JavaRuntimeHelper::TaggedValues => tagged_members(),
        JavaRuntimeHelper::CheckedIntegers => runtime_methods(helper),
        JavaRuntimeHelper::FloatBits => runtime_methods(helper),
        JavaRuntimeHelper::Unicode => runtime_methods(helper),
        JavaRuntimeHelper::Bytes => bytes_members(),
        JavaRuntimeHelper::ImmutableLists => runtime_methods(helper),
        JavaRuntimeHelper::StringOperations => runtime_methods(helper),
        JavaRuntimeHelper::Interfaces => vec![static_method(
            vec![],
            JavaType::primitive(JavaPrimitive::Boolean),
            "interfaceSupport",
            vec![],
            vec![JavaStmt::Return(Some(bool_literal(true)))],
        )],
    };
    vec![JavaFileItem::RuntimeMembers { helper, members }]
}

#[cfg(test)]
#[path = "tests/runtime.rs"]
mod tests;
