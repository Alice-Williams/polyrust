//! Java private access belongs to a top-level nest, not a source file.

use super::{
    JavaConstructorRef, JavaDeclarationKind, JavaExprKind, JavaFieldRef, JavaFileItem, JavaMember,
    JavaMethodDeclaration, JavaModifier, JavaType, JavaTypeName, JavaVisibility,
};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedSymbolId, TargetAstContext, TargetSymbolRef};
use portable_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn verify(
    item: &JavaFileItem,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut declarations = BTreeMap::new();
    let mut constructors = Vec::new();
    for file in context.files() {
        for item in file.items() {
            if let JavaFileItem::Type { declaration, .. } = item {
                let mut pending = vec![(declaration, false, false)];
                while let Some((node, parent_private, nested)) = pending.pop() {
                    let private = parent_private || node.visibility == JavaVisibility::Private;
                    if let Some(id) = node.declared {
                        declarations.insert(GeneratedSymbolId::Type(id), private);
                        for member in &node.members {
                            if let JavaMember::Constructor(constructor) = member {
                                constructors.push((id, node, constructor, nested));
                            }
                        }
                    }
                    for member in &node.members {
                        let (symbol, member_private) = match member {
                            JavaMember::NestedType(child) => {
                                pending.push((child, private, true));
                                continue;
                            }
                            JavaMember::Field(field) => (
                                field.declared.map(GeneratedSymbolId::Value),
                                field.modifiers.contains(&JavaModifier::Private),
                            ),
                            JavaMember::EnumConstant(value) => {
                                (Some(GeneratedSymbolId::Value(value.declared)), false)
                            }
                            JavaMember::Method(method) => (
                                match method.declared {
                                    JavaMethodDeclaration::Callable(id) => {
                                        Some(GeneratedSymbolId::Callable(id))
                                    }
                                    JavaMethodDeclaration::Interface(id) => {
                                        Some(GeneratedSymbolId::InterfaceMethod(id))
                                    }
                                    _ => None,
                                },
                                method.modifiers.contains(&JavaModifier::Private),
                            ),
                            JavaMember::Constructor(_) | JavaMember::CompileFailField(_) => {
                                (None, false)
                            }
                        };
                        if let Some(symbol) = symbol {
                            declarations.insert(symbol, private || member_private);
                        }
                    }
                }
            }
        }
    }
    let local = item.declared_symbols().into_iter().collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for symbol in item.symbols() {
        if let TargetSymbolRef::Generated(id) = symbol
            && !local.contains(&id)
            && declarations.get(&id) == Some(&true)
        {
            violations.push(error(
                "Java private generated symbol is referenced outside its top-level nest",
            ));
        }
    }
    let members = match item {
        JavaFileItem::Type { declaration, .. } => &declaration.members,
        JavaFileItem::RuntimeMembers { members, .. } => members,
    };
    for member in members {
        super::expression_visit::member(member, &mut |value| {
            if let JavaExprKind::Field { receiver, field } = &value.kind {
                let private_owner = match field {
                    JavaFieldRef::Generated { owner, .. } => Some(JavaTypeName::Generated(*owner)),
                    JavaFieldRef::Structural { name, .. } => {
                        let private = super::field_metadata::find_type_declaration(&receiver.ty, context).is_some_and(|node| {
                            node.record_components.iter().any(|value| value.name == *name)
                                || node.members.iter().any(|member| matches!(member, JavaMember::Field(value) if value.name == *name && value.modifiers.contains(&JavaModifier::Private)))
                        });
                        match &receiver.ty {
                            JavaType::Reference(owner) | JavaType::Generic { raw: owner, .. }
                                if private =>
                            {
                                Some(owner.clone())
                            }
                            _ => None,
                        }
                    }
                    JavaFieldRef::Known(_) => None,
                };
                let same_nest = match private_owner {
                    Some(JavaTypeName::Generated(owner)) => {
                        local.contains(&GeneratedSymbolId::Type(owner))
                    }
                    Some(JavaTypeName::Known(_)) => {
                        matches!(item, JavaFileItem::RuntimeMembers { .. })
                    }
                    None => true,
                };
                if !same_nest {
                    violations.push(error(
                        "Java private instance field is referenced outside its top-level nest",
                    ));
                }
            }
            if let JavaExprKind::New {
                constructor: JavaConstructorRef::Generated { owner, parameters },
                ..
            } = &value.kind
            {
                for (id, node, constructor, nested) in &constructors {
                    if id != owner
                        || !constructor
                            .parameters
                            .iter()
                            .map(|parameter| &parameter.ty)
                            .eq(parameters.iter())
                    {
                        continue;
                    }
                    if *nested
                        && node.kind == JavaDeclarationKind::FinalClass
                        && !node.modifiers.contains(&JavaModifier::Static)
                    {
                        violations.push(error("Java generated construction requires a static nested class or an implicit-static nominal"));
                    }
                    if constructor.modifiers.contains(&JavaModifier::Private)
                        && !local.contains(&GeneratedSymbolId::Type(*owner))
                    {
                        violations.push(error(
                            "Java private constructor is referenced outside its top-level nest",
                        ));
                    }
                }
            }
        });
    }
    violations
}

fn error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::InvalidStructure, message)
}
