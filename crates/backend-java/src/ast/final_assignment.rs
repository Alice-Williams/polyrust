//! Java AST: final assignment.

use super::blocks::JavaBlock;
use super::constructor_flow::analyze_constructor_final_assignment_statement;
use super::declaration_model::{JavaMember, JavaModifier, JavaTypeDeclaration};
use super::expression_model::JavaValueRef;
use super::expression_nodes::{JavaExpr, JavaExprKind, JavaFieldRef};
use super::identifiers::JavaIdentifier;
use super::statement_model::JavaStmt;
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) type JavaFinalAssignmentState = BTreeSet<JavaIdentifier>;
pub(super) type JavaFinalAssignmentStates = BTreeSet<JavaFinalAssignmentState>;

#[derive(Default)]
pub(super) struct JavaFinalAssignmentOutcome {
    pub(super) fallthrough: JavaFinalAssignmentStates,
    pub(super) constructor_returns: JavaFinalAssignmentStates,
}

pub(super) fn declaration_blank_instance_finals(
    declaration: &JavaTypeDeclaration,
) -> BTreeSet<JavaIdentifier> {
    let mut fields = declaration
        .record_components
        .iter()
        .map(|component| component.name.clone())
        .collect::<BTreeSet<_>>();
    fields.extend(
        declaration
            .members
            .iter()
            .filter_map(|member| match member {
                JavaMember::Field(field)
                    if !field.modifiers.contains(&JavaModifier::Static)
                        && field.modifiers.contains(&JavaModifier::Final)
                        && field.initializer.is_none() =>
                {
                    Some(field.name.clone())
                }
                _ => None,
            }),
    );
    fields
}

pub(super) fn assigned_blank_final<'a>(
    target: &'a JavaExpr,
    blank_finals: &BTreeSet<JavaIdentifier>,
) -> Option<&'a JavaIdentifier> {
    let JavaExprKind::Field { receiver, field } = &target.kind else {
        return None;
    };
    if !matches!(receiver.kind, JavaExprKind::Value(JavaValueRef::This)) {
        return None;
    }
    let name = match field {
        JavaFieldRef::Structural { name, .. } | JavaFieldRef::Generated { name, .. } => name,
        JavaFieldRef::Known(_) => return None,
    };
    blank_finals.contains(name).then_some(name)
}

pub(super) fn collect_blank_final_reads(
    value: &JavaExpr,
    blank_finals: &BTreeSet<JavaIdentifier>,
    reads: &mut BTreeSet<JavaIdentifier>,
) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match &value.kind {
            JavaExprKind::Literal(_) | JavaExprKind::Value(_) | JavaExprKind::Lambda { .. } => {}
            JavaExprKind::Unary { operand, .. } => pending.push(operand),
            JavaExprKind::Binary { left, right, .. } => {
                pending.push(left);
                pending.push(right);
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } => {
                pending.push(condition);
                pending.push(when_true);
                pending.push(when_false);
            }
            JavaExprKind::Call {
                receiver,
                arguments,
                ..
            } => {
                pending.extend(receiver.iter().map(Box::as_ref));
                pending.extend(arguments);
            }
            JavaExprKind::New { arguments, .. } => pending.extend(arguments),
            JavaExprKind::NewArray { length, .. } => pending.push(length),
            JavaExprKind::ArrayIndex { array, index } => {
                pending.push(array);
                pending.push(index);
            }
            JavaExprKind::Field { receiver, field } => {
                pending.push(receiver);
                if matches!(receiver.kind, JavaExprKind::Value(JavaValueRef::This)) {
                    let name = match field {
                        JavaFieldRef::Structural { name, .. }
                        | JavaFieldRef::Generated { name, .. } => Some(name),
                        JavaFieldRef::Known(_) => None,
                    };
                    if let Some(name) = name
                        && blank_finals.contains(name)
                    {
                        reads.insert(name.clone());
                    }
                }
            }
            JavaExprKind::Cast { value, .. }
            | JavaExprKind::InterfaceCoercion { value, .. }
            | JavaExprKind::ArrayOwnershipTransition { value, .. }
            | JavaExprKind::InstanceOf { value, .. } => pending.push(value),
        }
    }
}

fn statement_blank_final_reads(
    statement: &JavaStmt,
    blank_finals: &BTreeSet<JavaIdentifier>,
) -> BTreeSet<JavaIdentifier> {
    let mut reads = BTreeSet::new();
    let mut collect = |value: &JavaExpr| {
        collect_blank_final_reads(value, blank_finals, &mut reads);
    };
    match statement {
        JavaStmt::Local { value, .. } | JavaStmt::Return(value) => {
            if let Some(value) = value {
                collect(value);
            }
        }
        JavaStmt::Assign { target, value } => {
            match &target.kind {
                JavaExprKind::Value(JavaValueRef::Local(_)) => {}
                JavaExprKind::Field { receiver, .. } => collect(receiver),
                JavaExprKind::ArrayIndex { array, index } => {
                    collect(array);
                    collect(index);
                }
                _ => collect(target),
            }
            collect(value);
        }
        JavaStmt::Expression(value) | JavaStmt::Throw(value) | JavaStmt::ThrowAssertion(value) => {
            collect(value)
        }
        JavaStmt::If { condition, .. } | JavaStmt::While { condition, .. } => {
            collect(condition);
        }
        JavaStmt::ForEach { iterable, .. } => collect(iterable),
        JavaStmt::Switch { value, .. } => collect(value),
        JavaStmt::TryCatch { .. } | JavaStmt::Break | JavaStmt::Continue => {}
    }
    reads
}

pub(super) fn report_unassigned_blank_final_reads(
    statement: &JavaStmt,
    incoming: &JavaFinalAssignmentStates,
    blank_finals: &BTreeSet<JavaIdentifier>,
    violations: &mut Vec<AstViolation>,
) {
    for field in statement_blank_final_reads(statement, blank_finals) {
        if incoming.iter().any(|state| !state.contains(&field)) {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                format!(
                    "blank final Java field `{}` is read before it is definitely assigned",
                    field.as_str()
                ),
            ));
        }
    }
}

pub(super) fn block_assigns_blank_final(
    block: &JavaBlock,
    blank_finals: &BTreeSet<JavaIdentifier>,
) -> bool {
    block.statements.iter().any(|statement| match statement {
        JavaStmt::Assign { target, .. } => assigned_blank_final(target, blank_finals).is_some(),
        JavaStmt::If {
            then_block,
            else_block,
            ..
        } => {
            block_assigns_blank_final(then_block, blank_finals)
                || else_block
                    .as_ref()
                    .is_some_and(|block| block_assigns_blank_final(block, blank_finals))
        }
        JavaStmt::ForEach { body, .. } | JavaStmt::While { body, .. } => {
            block_assigns_blank_final(body, blank_finals)
        }
        JavaStmt::Switch { arms, .. } => arms
            .iter()
            .any(|arm| block_assigns_blank_final(&arm.body, blank_finals)),
        JavaStmt::TryCatch { try_block, catches } => {
            block_assigns_blank_final(try_block, blank_finals)
                || catches
                    .iter()
                    .any(|catch| block_assigns_blank_final(&catch.body, blank_finals))
        }
        JavaStmt::Local { .. }
        | JavaStmt::Expression(_)
        | JavaStmt::Return(_)
        | JavaStmt::Throw(_)
        | JavaStmt::ThrowAssertion(_)
        | JavaStmt::Break
        | JavaStmt::Continue => false,
    })
}

pub(super) fn analyze_constructor_final_assignments(
    block: &JavaBlock,
    incoming: JavaFinalAssignmentStates,
    blank_finals: &BTreeSet<JavaIdentifier>,
    violations: &mut Vec<AstViolation>,
) -> JavaFinalAssignmentOutcome {
    let mut fallthrough = incoming;
    let mut constructor_returns = JavaFinalAssignmentStates::new();
    for statement in &block.statements {
        if fallthrough.is_empty() {
            break;
        }
        let outcome = analyze_constructor_final_assignment_statement(
            statement,
            fallthrough,
            blank_finals,
            violations,
        );
        fallthrough = outcome.fallthrough;
        constructor_returns.extend(outcome.constructor_returns);
    }
    JavaFinalAssignmentOutcome {
        fallthrough,
        constructor_returns,
    }
}
