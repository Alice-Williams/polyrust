//! Fail-closed grammar admission for the first scalar/record/shared-local profile.

use crate::ast::{
    CAggregateRef, CBlock, CCallableKind, CDeclarationKind, CDefinitionKind, CEffect, CFileItem,
    CFunctionRef, CInitializer, CInitializerKind, CLinkage, CObjectType, CObjectTypeKind, CPlace,
    CPlaceKind, CPointerTarget, CReturnType, CScalarType, CSourceFile, CStatement, CStatementKind,
    CValue,
};

#[path = "profile_constants.rs"]
mod constants;
#[path = "profile_inventory.rs"]
mod inventory;
#[path = "profile_layout.rs"]
mod layout;
#[path = "profile_values.rs"]
mod values;

#[derive(Clone, Copy)]
pub(super) enum Node<'a> {
    Item(&'a CFileItem),
    Block(&'a CBlock),
    Statement(&'a CStatement),
    Value(&'a CValue),
    Effect(&'a CEffect),
    Place(&'a CPlace),
    Initializer(&'a CInitializer),
    Type(&'a CObjectType),
}

fn scalar(ty: &CObjectType) -> bool {
    matches!(
        ty.kind(),
        CObjectTypeKind::Scalar(
            CScalarType::I32
                | CScalarType::I64
                | CScalarType::U32
                | CScalarType::Int
                | CScalarType::Bool
                | CScalarType::F64
        )
    )
}

pub(super) fn ordered_sources(sources: &[CSourceFile]) -> Result<Vec<&CSourceFile>, String> {
    Ok(layout::Layout::new(sources)?.ordered())
}

#[cfg(test)]
pub(super) fn check_package(sources: &[CSourceFile]) -> Result<(), String> {
    walk_package(sources, |_, _, _| Ok(()))
}

pub(super) fn check_registered_package(
    registry: &crate::ast::CRegistry,
    sources: &[CSourceFile],
) -> Result<(), String> {
    walk(sources, Some(registry), |_, _, _| Ok(()))
}

#[cfg(test)]
pub(super) fn walk_package<'a>(
    sources: &'a [CSourceFile],
    visit: impl FnMut(&'a CSourceFile, Node<'a>, usize) -> Result<(), String>,
) -> Result<(), String> {
    walk(sources, None, visit)
}

pub(super) fn walk_registered_package<'a>(
    registry: &crate::ast::CRegistry,
    sources: &'a [CSourceFile],
    visit: impl FnMut(&'a CSourceFile, Node<'a>, usize) -> Result<(), String>,
) -> Result<(), String> {
    walk(sources, Some(registry), visit)
}

fn walk<'a>(
    sources: &'a [CSourceFile],
    registry: Option<&crate::ast::CRegistry>,
    mut visit: impl FnMut(&'a CSourceFile, Node<'a>, usize) -> Result<(), String>,
) -> Result<(), String> {
    if let Some(registry) = registry {
        super::source_package::check(registry, sources)?;
    }
    let layout = layout::Layout::new(sources)?;
    let exports = registry
        .map(super::constant_exports::collect)
        .transpose()?
        .unwrap_or_default();
    let has_foreign_exports = !exports.foreign.is_empty();
    if !layout.has_public_declaration() && !has_foreign_exports {
        return Err("C public header requires an exported function, scalar constant or certified foreign constant".into());
    }
    let mut pending: Vec<_> = layout
        .ordered()
        .into_iter()
        .rev()
        .flat_map(|source| {
            source
                .items()
                .iter()
                .rev()
                .map(move |item| (source, Node::Item(item), 0usize))
        })
        .collect();
    let mut nodes = 0usize;
    let mut functions = 0usize;
    let mut objects = 0usize;
    let mut inventory = inventory::Inventory::with_registry(registry)?;
    while let Some((source, node, depth)) = pending.pop() {
        nodes += 1;
        // Bound recursive downstream checks BEFORE entering them. These are
        // conservative verifier budgets, not generic language arity limits.
        if depth > 128 || nodes > 100_000 {
            return Err("C projection verifier budget exceeded".into());
        }
        visit(source, node, depth)?;
        inventory.visit(node)?;
        let mut add = |child| pending.push((source, child, depth + 1));
        match node {
            Node::Item(CFileItem::Comment(_)) => {}
            Node::Item(CFileItem::Declaration(declaration)) => {
                match declaration.kind() {
                    CDeclarationKind::Aggregate {
                        owner: CAggregateRef::Struct(record),
                        members,
                    } => {
                        if source.identity().key().role
                            == crate::ast::CFileRole::GeneratedPublicHeader
                            && !crate::ownership::value_transport::scalar_result(
                                registry,
                                &CObjectType::structure(record.clone()),
                            )
                        {
                            return Err("C public record requires the exact registered scalar-result layout".into());
                        }
                        for member in members {
                            if !scalar(member.ty()) {
                                return Err("first C profile requires scalar record fields".into());
                            }
                            add(Node::Type(member.ty()));
                        }
                    }
                    CDeclarationKind::FunctionPrototype {
                        function,
                        linkage: linkage @ (CLinkage::External | CLinkage::Internal),
                    } => {
                        signature(
                            function,
                            if *linkage == CLinkage::Internal
                                || function.file().key().role
                                    == crate::ast::CFileRole::GeneratedPublicHeader
                            {
                                registry
                            } else {
                                None
                            },
                        )?;
                    }
                    CDeclarationKind::ObjectDeclaration(object) => {
                        constants::object(object)?;
                        add(Node::Type(object.ty()));
                    }
                    _ => return Err("C declaration is outside the first shared profile".into()),
                }
            }
            Node::Item(CFileItem::Definition(definition)) => match definition.kind() {
                CDefinitionKind::Function {
                    function,
                    linkage: linkage @ (CLinkage::External | CLinkage::Internal),
                    body,
                    ..
                } => {
                    signature(
                        function,
                        if *linkage == CLinkage::Internal
                            || function.file().key().role
                                == crate::ast::CFileRole::GeneratedPublicHeader
                        {
                            registry
                        } else {
                            None
                        },
                    )?;
                    functions += 1;
                    add(Node::Block(body));
                }
                CDefinitionKind::Object {
                    object,
                    linkage: CLinkage::External,
                    initializer,
                } => {
                    constants::initializer(object, initializer)?;
                    objects += 1;
                    add(Node::Type(object.ty()));
                    add(Node::Initializer(initializer));
                }
                _ => return Err("C definition is outside the first shared profile".into()),
            },
            Node::Item(CFileItem::StaticAssert(assertion)) => {
                // Closed, fixed-depth assertion grammar; not executable source.
                super::platform::classify(assertion)?;
            }
            Node::Block(block) => {
                for statement in block.statements() {
                    add(Node::Statement(statement));
                }
            }
            Node::Statement(statement) => match statement.kind() {
                CStatementKind::Block(block) => add(Node::Block(block)),
                CStatementKind::Declare(local) => {
                    add(Node::Type(local.local().ty()));
                    add(Node::Initializer(
                        local
                            .initializer()
                            .ok_or("first C profile requires initialized locals")?,
                    ));
                }
                CStatementKind::Assign { place, value }
                    if matches!(place.kind(), CPlaceKind::Local(_))
                        && matches!(
                            place.ty().kind(),
                            CObjectTypeKind::Scalar(CScalarType::Bool)
                        ) =>
                {
                    add(Node::Place(place));
                    add(Node::Value(value));
                }
                CStatementKind::Evaluate(effect) => add(Node::Effect(effect)),
                CStatementKind::Return(None) => {}
                CStatementKind::Discard(value) | CStatementKind::Return(Some(value)) => {
                    add(Node::Value(value))
                }
                CStatementKind::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    add(Node::Value(condition));
                    add(Node::Block(then_block));
                    add(Node::Block(else_block));
                }
                _ => return Err("C statement is outside the first shared profile".into()),
            },
            Node::Initializer(initializer) => {
                add(Node::Type(initializer.ty()));
                match initializer.kind() {
                    CInitializerKind::Expression(value) => add(Node::Value(value)),
                    CInitializerKind::Struct { members, .. } => {
                        for (_, value) in members {
                            add(Node::Initializer(value));
                        }
                    }
                    _ => return Err("C initializer is outside the first shared profile".into()),
                }
            }
            Node::Effect(effect) => {
                let call = effect.call();
                let CCallableKind::Direct(function) = call.callable().kind() else {
                    return Err("C shared effect profile requires resolved direct calls".into());
                };
                signature(function, registry)?;
                if !matches!(function.signature().return_type(), CReturnType::Void) {
                    return Err("C effect call requires a void result".into());
                }
                for argument in call.arguments() {
                    add(Node::Value(argument));
                }
            }
            Node::Value(value) => values::visit(value, registry, &mut add)?,
            Node::Place(place) => {
                add(Node::Type(place.ty()));
                match place.kind() {
                    CPlaceKind::Local(_) | CPlaceKind::Parameter(_) => {}
                    CPlaceKind::Global(object) => constants::object(object)?,
                    CPlaceKind::Member { base, .. } => add(Node::Place(base)),
                    CPlaceKind::Dereference(value) => add(Node::Value(value)),
                    _ => return Err("C place is outside the first shared profile".into()),
                }
            }
            Node::Type(ty) => match ty.kind() {
                CObjectTypeKind::Scalar(
                    CScalarType::I32
                    | CScalarType::I64
                    | CScalarType::U32
                    | CScalarType::U64
                    | CScalarType::Int
                    | CScalarType::Bool
                    | CScalarType::F64,
                )
                | CObjectTypeKind::Struct(_) => {}
                CObjectTypeKind::Pointer(CPointerTarget::Object(pointee)) => {
                    add(Node::Type(pointee))
                }
                _ => return Err("C type is outside the first shared profile".into()),
            },
        }
    }
    if functions == 0 && objects == 0 && !has_foreign_exports {
        return Err("C shared profile requires a function or scalar constant definition".into());
    }
    inventory.finish()?;
    Ok(())
}

fn signature(
    function: &CFunctionRef,
    registry: Option<&crate::ast::CRegistry>,
) -> Result<(), String> {
    let admitted = |ty: &CObjectType| {
        scalar(ty)
            || (matches!(ty.kind(), CObjectTypeKind::Struct(record)
            if (record.file() == function.file()
                && matches!(record.file().key().role, crate::ast::CFileRole::GeneratedSource | crate::ast::CFileRole::TestSource))
                || (record.file().key().role == crate::ast::CFileRole::GeneratedPublicHeader
                    && (function.file() == record.file()
                        || function.file().key().role == crate::ast::CFileRole::GeneratedSource
                        || registry.is_some_and(|registry| registry.imported_struct(record).is_ok()))))
                && crate::ownership::value_transport::scalar_result(registry, ty))
    };
    let admitted_result = match function.signature().return_type() {
        CReturnType::Void => true,
        CReturnType::Value(result) => admitted(result.declared_type()),
    };
    if !admitted_result
        || function
            .signature()
            .parameters()
            .iter()
            .any(|parameter| !admitted(parameter.declared_type()))
    {
        return Err(
            "first C shared profile admits scalar parameters and scalar/void returns".into(),
        );
    }
    Ok(())
}
