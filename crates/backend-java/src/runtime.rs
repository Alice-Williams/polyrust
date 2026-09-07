use crate::{ast::*, dialect::*};

mod core_types;
use core_types::*;
mod equality;
use equality::*;
mod tagged_values;
use tagged_values::*;
mod option;
use option::*;
mod value_result;
use value_result::*;
mod bytes_storage;
use bytes_storage::*;
mod dispatch;
use dispatch::*;
mod string_replace;
use string_replace::*;
mod string_transform;
use string_transform::*;
mod unicode;
use unicode::*;
mod unicode_validation;
use unicode_validation::*;
mod checked_integer;
use checked_integer::*;
mod float_list;
use float_list::*;
mod bytes_operations;
use bytes_operations::*;
mod declaration_builders;
use declaration_builders::*;
mod expression_builders;
use expression_builders::*;
mod call_builders;
use call_builders::*;

pub(crate) fn shell_item() -> JavaFileItem {
    JavaFileItem::Type {
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
