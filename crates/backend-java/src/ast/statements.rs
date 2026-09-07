//! Java AST: statements.

use super::exceptions::{
    admitted_throwable_is_supertype_of, block_checked_exceptions, is_checked_exception,
    throwable_known_type,
};
use super::operator_signatures::invocation_types_match;
use super::statement_model::{JavaPattern, JavaStmt};
use super::switch_patterns::{
    is_java_statement_expression, iterable_element_type, verify_switch_patterns,
};
use super::types::{JavaPrimitive, JavaType, JavaTypeUse, type_error};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedSymbolId, TargetAstContext, TargetSymbolRef};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

impl JavaStmt {
    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        match self {
            Self::Local { ty, value, .. } => {
                ty.symbols(symbols);
                if let Some(value) = value {
                    value.symbols(symbols);
                }
            }
            Self::Assign { target, value } => {
                target.symbols(symbols);
                value.symbols(symbols);
            }
            Self::Expression(value) | Self::Throw(value) | Self::ThrowAssertion(value) => {
                value.symbols(symbols)
            }
            Self::Return(value) => {
                if let Some(value) = value {
                    value.symbols(symbols);
                }
            }
            Self::If {
                condition,
                then_block,
                else_block,
            } => {
                condition.symbols(symbols);
                then_block.symbols(symbols);
                if let Some(block) = else_block {
                    block.symbols(symbols);
                }
            }
            Self::ForEach {
                binding_type,
                iterable,
                body,
                ..
            } => {
                binding_type.symbols(symbols);
                iterable.symbols(symbols);
                body.symbols(symbols);
            }
            Self::While { condition, body } => {
                condition.symbols(symbols);
                body.symbols(symbols);
            }
            Self::Switch { value, arms } => {
                value.symbols(symbols);
                for arm in arms {
                    match &arm.pattern {
                        JavaPattern::Type { ty, .. } => ty.symbols(symbols),
                        JavaPattern::EnumVariant {
                            enumeration,
                            variant,
                        } => {
                            symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(
                                *enumeration,
                            )));
                            symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Value(
                                *variant,
                            )));
                        }
                        JavaPattern::Default | JavaPattern::Literal(_) => {}
                    }
                    arm.body.symbols(symbols);
                }
            }
            Self::TryCatch { try_block, catches } => {
                try_block.symbols(symbols);
                for catch in catches {
                    catch.exception_type.symbols(symbols);
                    catch.body.symbols(symbols);
                }
            }
            Self::Break | Self::Continue => {}
        }
    }

    pub fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        let mut violations = Vec::new();
        match self {
            Self::Local { ty, value, .. } => {
                violations.extend(ty.verify(JavaTypeUse::Value));
                if let Some(value) = value {
                    violations.extend(value.verify(context));
                    if &value.ty != ty {
                        violations.push(type_error("local initializer type mismatch"));
                    }
                }
            }
            Self::Assign { target, value } => {
                violations.extend(target.verify(context));
                violations.extend(value.verify(context));
                if target.ty != value.ty {
                    violations.push(type_error("assignment type mismatch"));
                }
            }
            Self::Expression(value) | Self::Throw(value) | Self::ThrowAssertion(value) => {
                violations.extend(value.verify(context));
                if matches!(self, Self::Expression(_)) && !is_java_statement_expression(value) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java expression statement must be a method invocation or class instance creation",
                    ));
                }
                if matches!(self, Self::Throw(_)) && throwable_known_type(&value.ty).is_none() {
                    violations.push(type_error(
                        "Java throw expression must have a known throwable type",
                    ));
                }
            }
            Self::Return(value) => {
                if let Some(value) = value {
                    violations.extend(value.verify(context));
                }
            }
            Self::If {
                condition,
                then_block,
                else_block,
            } => {
                violations.extend(condition.verify(context));
                if condition.ty != JavaType::Primitive(JavaPrimitive::Boolean) {
                    violations.push(type_error("if condition must be boolean"));
                }
                violations.extend(then_block.verify(context));
                if let Some(block) = else_block {
                    violations.extend(block.verify(context));
                }
            }
            Self::ForEach {
                binding_type,
                iterable,
                body,
                ..
            } => {
                violations.extend(binding_type.verify(JavaTypeUse::Value));
                violations.extend(iterable.verify(context));
                match iterable_element_type(&iterable.ty) {
                    Some(element) if invocation_types_match(binding_type, element) => {}
                    Some(_) => violations.push(type_error(
                        "Java foreach binding type does not match its iterable element type",
                    )),
                    None => violations.push(type_error(
                        "Java foreach expression must be an array or typed List",
                    )),
                }
                violations.extend(body.verify(context));
            }
            Self::While { condition, body } => {
                violations.extend(condition.verify(context));
                if condition.ty != JavaType::Primitive(JavaPrimitive::Boolean) {
                    violations.push(type_error("while condition must be boolean"));
                }
                violations.extend(body.verify(context));
            }
            Self::Switch { value, arms } => {
                violations.extend(value.verify(context));
                violations.extend(verify_switch_patterns(value, arms, context));
                for arm in arms {
                    violations.extend(arm.body.verify(context));
                }
            }
            Self::TryCatch { try_block, catches } => {
                violations.extend(try_block.verify(context));
                if catches.is_empty() {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "typed try statement requires at least one catch",
                    ));
                }
                let thrown = block_checked_exceptions(try_block, context);
                let mut caught_types = BTreeSet::new();
                let mut prior_caught_types = Vec::new();
                for catch in catches {
                    violations.extend(catch.exception_type.verify(JavaTypeUse::Parameter));
                    match throwable_known_type(&catch.exception_type) {
                        Some(caught) => {
                            if prior_caught_types.iter().any(|prior| {
                                *prior != caught
                                    && admitted_throwable_is_supertype_of(*prior, caught)
                            }) {
                                violations.push(AstViolation::new(
                                    DiagnosticCode::InvalidControlFlow,
                                    "Java catch clause is dominated by an earlier admitted throwable supertype",
                                ));
                            }
                            if !caught_types.insert(caught) {
                                violations.push(AstViolation::new(
                                    DiagnosticCode::DuplicateDeclaration,
                                    "Java catch type is repeated",
                                ));
                            }
                            if is_checked_exception(caught) && !thrown.contains(&caught) {
                                violations.push(AstViolation::new(
                                    DiagnosticCode::InvalidStructure,
                                    "Java checked catch cannot be reached from its try block",
                                ));
                            }
                            prior_caught_types.push(caught);
                        }
                        None => violations.push(type_error(
                            "Java catch parameter must have a known throwable type",
                        )),
                    }
                    violations.extend(catch.body.verify(context));
                }
            }
            Self::Break | Self::Continue => {}
        }
        violations
    }
}
