//! Java AST: constructor flow.

use super::declaration_model::{JavaConstructor, JavaTypeDeclaration};
use super::final_assignment::{
    JavaFinalAssignmentOutcome, JavaFinalAssignmentState, JavaFinalAssignmentStates,
    analyze_constructor_final_assignments, assigned_blank_final, block_assigns_blank_final,
    declaration_blank_instance_finals, report_unassigned_blank_final_reads,
};
use super::identifiers::JavaIdentifier;
use super::statement_model::{JavaPattern, JavaStmt};
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn analyze_constructor_final_assignment_statement(
    statement: &JavaStmt,
    incoming: JavaFinalAssignmentStates,
    blank_finals: &BTreeSet<JavaIdentifier>,
    violations: &mut Vec<AstViolation>,
) -> JavaFinalAssignmentOutcome {
    report_unassigned_blank_final_reads(statement, &incoming, blank_finals, violations);
    match statement {
        JavaStmt::Assign { target, .. } => {
            let mut fallthrough = JavaFinalAssignmentStates::new();
            if let Some(field) = assigned_blank_final(target, blank_finals) {
                let duplicate = incoming.iter().any(|state| state.contains(field));
                if duplicate {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        format!(
                            "blank final Java field `{}` can be assigned more than once on a constructor path",
                            field.as_str()
                        ),
                    ));
                }
                for mut state in incoming {
                    state.insert(field.clone());
                    fallthrough.insert(state);
                }
            } else {
                fallthrough = incoming;
            }
            JavaFinalAssignmentOutcome {
                fallthrough,
                ..JavaFinalAssignmentOutcome::default()
            }
        }
        JavaStmt::Return(_) => JavaFinalAssignmentOutcome {
            constructor_returns: incoming,
            ..JavaFinalAssignmentOutcome::default()
        },
        JavaStmt::Throw(_) | JavaStmt::ThrowAssertion(_) => JavaFinalAssignmentOutcome::default(),
        JavaStmt::If {
            then_block,
            else_block,
            ..
        } => {
            let then_outcome = analyze_constructor_final_assignments(
                then_block,
                incoming.clone(),
                blank_finals,
                violations,
            );
            let else_outcome = if let Some(block) = else_block {
                analyze_constructor_final_assignments(block, incoming, blank_finals, violations)
            } else {
                JavaFinalAssignmentOutcome {
                    fallthrough: incoming,
                    ..JavaFinalAssignmentOutcome::default()
                }
            };
            JavaFinalAssignmentOutcome {
                fallthrough: then_outcome
                    .fallthrough
                    .union(&else_outcome.fallthrough)
                    .cloned()
                    .collect(),
                constructor_returns: then_outcome
                    .constructor_returns
                    .union(&else_outcome.constructor_returns)
                    .cloned()
                    .collect(),
            }
        }
        JavaStmt::ForEach { body, .. } | JavaStmt::While { body, .. } => {
            if block_assigns_blank_final(body, blank_finals) {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidControlFlow,
                    "blank final Java fields cannot be assigned in a portable constructor loop",
                ));
            }
            let body_outcome = analyze_constructor_final_assignments(
                body,
                incoming.clone(),
                blank_finals,
                violations,
            );
            JavaFinalAssignmentOutcome {
                fallthrough: incoming,
                constructor_returns: body_outcome.constructor_returns,
            }
        }
        JavaStmt::Switch { arms, .. } => {
            let mut outcome = JavaFinalAssignmentOutcome::default();
            for arm in arms {
                let arm_outcome = analyze_constructor_final_assignments(
                    &arm.body,
                    incoming.clone(),
                    blank_finals,
                    violations,
                );
                outcome.fallthrough.extend(arm_outcome.fallthrough);
                outcome
                    .constructor_returns
                    .extend(arm_outcome.constructor_returns);
            }
            if !arms
                .iter()
                .any(|arm| matches!(arm.pattern, JavaPattern::Default))
            {
                outcome.fallthrough.extend(incoming);
            }
            outcome
        }
        JavaStmt::TryCatch { try_block, catches } => {
            if block_assigns_blank_final(try_block, blank_finals)
                || catches
                    .iter()
                    .any(|catch| block_assigns_blank_final(&catch.body, blank_finals))
            {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidControlFlow,
                    "blank final Java assignment inside portable try/catch is not supported by sound definite-assignment analysis",
                ));
            }
            let mut outcome = analyze_constructor_final_assignments(
                try_block,
                incoming.clone(),
                blank_finals,
                violations,
            );
            for catch in catches {
                let catch_outcome = analyze_constructor_final_assignments(
                    &catch.body,
                    incoming.clone(),
                    blank_finals,
                    violations,
                );
                outcome.fallthrough.extend(catch_outcome.fallthrough);
                outcome
                    .constructor_returns
                    .extend(catch_outcome.constructor_returns);
            }
            outcome
        }
        JavaStmt::Break | JavaStmt::Continue => JavaFinalAssignmentOutcome::default(),
        JavaStmt::Local { .. } | JavaStmt::Expression(_) => JavaFinalAssignmentOutcome {
            fallthrough: incoming,
            ..JavaFinalAssignmentOutcome::default()
        },
    }
}

pub(super) fn verify_constructor_final_assignments(
    constructor: &JavaConstructor,
    declaration: &JavaTypeDeclaration,
) -> Vec<AstViolation> {
    let blank_finals = declaration_blank_instance_finals(declaration);
    if blank_finals.is_empty() {
        return vec![];
    }
    let mut initial = JavaFinalAssignmentStates::new();
    initial.insert(JavaFinalAssignmentState::new());
    let mut violations = Vec::new();
    let outcome = analyze_constructor_final_assignments(
        &constructor.body,
        initial,
        &blank_finals,
        &mut violations,
    );
    let normally_completing = outcome
        .fallthrough
        .union(&outcome.constructor_returns)
        .cloned()
        .collect::<JavaFinalAssignmentStates>();
    for state in normally_completing {
        for missing in blank_finals.difference(&state) {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                format!(
                    "Java constructor can complete normally without assigning blank final field `{}`",
                    missing.as_str()
                ),
            ));
        }
    }
    violations
}
