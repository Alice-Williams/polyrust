//! Fail-closed grammar admission for the first scalar/record/shared-local profile.

use crate::ast::{
    CAggregateRef, CBinaryOperator, CBlock, CCallableKind, CConversion, CDeclarationKind,
    CDefinitionKind, CFileItem, CFunctionRef, CInitializer, CInitializerKind, CLinkage, CLiteral,
    CObjectType, CObjectTypeKind, CPlace, CPlaceKind, CPointerTarget, CReturnType, CScalarType,
    CSignedLiteral, CSourceFile, CStatement, CStatementKind, CUnaryOperator, CValue, CValueKind,
};

#[path = "profile_inventory.rs"]
mod inventory;
#[path = "profile_layout.rs"]
mod layout;

#[derive(Clone, Copy)]
pub(super) enum Node<'a> {
    Item(&'a CFileItem),
    Block(&'a CBlock),
    Statement(&'a CStatement),
    Value(&'a CValue),
    Place(&'a CPlace),
    Initializer(&'a CInitializer),
    Type(&'a CObjectType),
}

fn scalar(ty: &CObjectType) -> bool {
    matches!(
        ty.kind(),
        CObjectTypeKind::Scalar(CScalarType::I32 | CScalarType::Int | CScalarType::Bool)
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
    let mut pending: Vec<_> = ordered_sources(sources)?
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
            Node::Item(CFileItem::Declaration(declaration)) => match declaration.kind() {
                CDeclarationKind::Aggregate {
                    owner: CAggregateRef::Struct(_),
                    members,
                } => {
                    for member in members {
                        if !scalar(member.ty()) {
                            return Err("first C profile requires scalar record fields".into());
                        }
                        add(Node::Type(member.ty()));
                    }
                }
                CDeclarationKind::FunctionPrototype {
                    function,
                    linkage: CLinkage::External | CLinkage::Internal,
                } => {
                    signature(function)?;
                }
                _ => return Err("C declaration is outside the first shared profile".into()),
            },
            Node::Item(CFileItem::Definition(definition)) => match definition.kind() {
                CDefinitionKind::Function {
                    function,
                    linkage: CLinkage::External | CLinkage::Internal,
                    body,
                    ..
                } => {
                    signature(function)?;
                    functions += 1;
                    add(Node::Block(body));
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
            Node::Value(value) => {
                add(Node::Type(value.ty()));
                match value.kind() {
                    CValueKind::Call(call) => {
                        let CCallableKind::Direct(function) = call.callable().kind() else {
                            return Err(
                                "C shared call profile requires resolved direct calls".into()
                            );
                        };
                        signature(function)?;
                        for argument in call.arguments() {
                            add(Node::Value(argument));
                        }
                    }
                    CValueKind::Literal(
                        CLiteral::Bool(_)
                        | CLiteral::Signed(CSignedLiteral::I32(_) | CSignedLiteral::Int(_)),
                    ) => {}
                    CValueKind::Read(place) | CValueKind::AddressOf(place) => {
                        add(Node::Place(place))
                    }
                    CValueKind::Unary {
                        operator: CUnaryOperator::LogicalNot,
                        operand,
                    } if matches!(
                        operand.ty().kind(),
                        CObjectTypeKind::Scalar(CScalarType::Bool)
                    ) =>
                    {
                        add(Node::Value(operand));
                    }
                    CValueKind::Binary {
                        operator,
                        left,
                        right,
                    } => {
                        if !matches!(
                            operator,
                            CBinaryOperator::Equal
                                | CBinaryOperator::NotEqual
                                | CBinaryOperator::Less
                                | CBinaryOperator::LessEqual
                                | CBinaryOperator::Greater
                                | CBinaryOperator::GreaterEqual
                        ) {
                            return Err(
                                "only scalar comparisons are admitted by the first C profile"
                                    .into(),
                            );
                        }
                        add(Node::Value(left));
                        add(Node::Value(right));
                    }
                    CValueKind::Convert {
                        conversion:
                            CConversion::Numeric(
                                CScalarType::Bool | CScalarType::Int | CScalarType::I32,
                            ),
                        operand,
                    } => add(Node::Value(operand)),
                    CValueKind::Convert {
                        conversion: CConversion::AddConst(ty),
                        operand,
                    } => {
                        add(Node::Type(ty));
                        add(Node::Value(operand));
                    }
                    _ => return Err("C expression is outside the first shared profile".into()),
                }
            }
            Node::Place(place) => {
                add(Node::Type(place.ty()));
                match place.kind() {
                    CPlaceKind::Local(_) | CPlaceKind::Parameter(_) => {}
                    CPlaceKind::Member { base, .. } => add(Node::Place(base)),
                    CPlaceKind::Dereference(value) => add(Node::Value(value)),
                    _ => return Err("C place is outside the first shared profile".into()),
                }
            }
            Node::Type(ty) => match ty.kind() {
                CObjectTypeKind::Scalar(
                    CScalarType::I32 | CScalarType::Int | CScalarType::Bool,
                )
                | CObjectTypeKind::Struct(_) => {}
                CObjectTypeKind::Pointer(CPointerTarget::Object(pointee)) => {
                    add(Node::Type(pointee))
                }
                _ => return Err("C type is outside the first shared profile".into()),
            },
        }
    }
    if functions == 0 {
        return Err("first C shared profile requires a function definition".into());
    }
    inventory.finish()?;
    Ok(())
}

fn signature(function: &CFunctionRef) -> Result<(), String> {
    let CReturnType::Value(result) = function.signature().return_type() else {
        return Err("first C profile requires scalar function returns".into());
    };
    if !scalar(result.declared_type())
        || function
            .signature()
            .parameters()
            .iter()
            .any(|parameter| !scalar(parameter.declared_type()))
    {
        return Err("first C shared profile admits only scalar parameters and returns".into());
    }
    Ok(())
}
