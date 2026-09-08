//! Control-flow reservations, including compiler-owned enum-switch helpers.

use super::{Code, expressions::expression};
use crate::ast::{JavaBlock, JavaPattern, JavaStmt};

pub(super) fn block(value: &JavaBlock) -> Code {
    let mut result = Code::default();
    for statement in &value.statements {
        let mut code = Code {
            bytes: 64,
            pool: 16,
            locals: 8,
            stack: 8,
            ..Code::default()
        };
        match statement {
            JavaStmt::Local { ty, value, .. } => {
                code.bytes = 32;
                code.ty(ty);
                if let Some(value) = value {
                    code.add(&expression(value));
                }
            }
            JavaStmt::Assign { target, value } => {
                code.bytes = 32;
                code.add(&expression(target));
                code.add(&expression(value));
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                code.bytes = 32;
                code.add(&expression(value));
            }
            JavaStmt::Return(value) => {
                code.bytes = 32;
                if let Some(value) = value {
                    code.add(&expression(value));
                }
            }
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                code.add(&expression(condition));
                code.add(&block(then_block));
                if let Some(body) = else_block {
                    code.add(&block(body));
                }
            }
            JavaStmt::ForEach {
                binding_type,
                iterable,
                body,
                ..
            } => {
                code.ty(binding_type);
                code.add(&expression(iterable));
                code.add(&block(body));
            }
            JavaStmt::While { condition, body } => {
                code.add(&expression(condition));
                code.add(&block(body));
            }
            JavaStmt::Switch { value, arms } => {
                code.add(&expression(value));
                let mut pattern_switch = false;
                for arm in arms {
                    code.bytes = code.bytes.saturating_add(64);
                    code.pool = code.pool.saturating_add(16);
                    code.locals = code.locals.saturating_add(2);
                    code.stack = code.stack.saturating_add(8);
                    match &arm.pattern {
                        JavaPattern::Default
                        | JavaPattern::Literal(_)
                        | JavaPattern::EnumVariant { .. } => {}
                        JavaPattern::Type { ty, .. } => {
                            pattern_switch = true;
                            code.ty(ty);
                        }
                    }
                    code.add(&block(&arm.body));
                }
                // A default-only enum switch still gets a javac map. Reserve
                // a possible map for every switch rather than infer enum-ness
                // from its labels (or dispatch on nominal ID spellings).
                code.helper.switches = code.helper.switches.saturating_add(1);
                code.helper.arms = code.helper.arms.saturating_add(arms.len());
                if pattern_switch {
                    code.bootstraps = code.bootstraps.saturating_add(1);
                    code.bootstrap_arguments = code.bootstrap_arguments.max(arms.len());
                    code.pool = code.pool.saturating_add(32);
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                code.add(&block(try_block));
                for catch in catches {
                    code.bytes = code.bytes.saturating_add(64);
                    code.pool = code.pool.saturating_add(16);
                    code.locals = code.locals.saturating_add(2);
                    code.exceptions = code.exceptions.saturating_add(1);
                    code.ty(&catch.exception_type);
                    code.add(&block(&catch.body));
                }
            }
            JavaStmt::Break | JavaStmt::Continue => {
                code.bytes = 8;
            }
        }
        result.add(&code);
    }
    result
}
