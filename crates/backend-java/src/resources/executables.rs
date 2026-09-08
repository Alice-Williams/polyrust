//! Exhaustive local, expression and reference type/name capacity checks.

use super::declarations::Checker;
use crate::ast::{
    JavaBlock, JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind, JavaFieldRef,
    JavaPattern, JavaStmt, JavaValueRef,
};

impl Checker<'_> {
    pub(super) fn block(&mut self, block: &JavaBlock) {
        for statement in &block.statements {
            match statement {
                JavaStmt::Local {
                    ty, name, value, ..
                } => {
                    self.ty(ty);
                    self.name(name);
                    if let Some(value) = value {
                        self.expression(value);
                    }
                }
                JavaStmt::Assign { target, value } => {
                    self.expression(target);
                    self.expression(value);
                }
                JavaStmt::Expression(value)
                | JavaStmt::Throw(value)
                | JavaStmt::ThrowAssertion(value) => self.expression(value),
                JavaStmt::Return(value) => {
                    if let Some(value) = value {
                        self.expression(value);
                    }
                }
                JavaStmt::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    self.expression(condition);
                    self.block(then_block);
                    if let Some(body) = else_block {
                        self.block(body);
                    }
                }
                JavaStmt::ForEach {
                    binding_type,
                    binding,
                    iterable,
                    body,
                } => {
                    self.ty(binding_type);
                    self.name(binding);
                    self.expression(iterable);
                    self.block(body);
                }
                JavaStmt::While { condition, body } => {
                    self.expression(condition);
                    self.block(body);
                }
                JavaStmt::Switch { value, arms } => {
                    self.expression(value);
                    for arm in arms {
                        match &arm.pattern {
                            JavaPattern::Type { ty, binding } => {
                                self.ty(ty);
                                self.name(binding);
                            }
                            JavaPattern::Default
                            | JavaPattern::Literal(_)
                            | JavaPattern::EnumVariant { .. } => {}
                        }
                        self.block(&arm.body);
                    }
                }
                JavaStmt::TryCatch { try_block, catches } => {
                    self.block(try_block);
                    for catch in catches {
                        self.ty(&catch.exception_type);
                        self.name(&catch.binding);
                        self.block(&catch.body);
                    }
                }
                JavaStmt::Break | JavaStmt::Continue => {}
            }
        }
    }

    pub(super) fn expression(&mut self, value: &JavaExpr) {
        self.ty(&value.ty);
        match &value.kind {
            JavaExprKind::Literal(_) => {}
            JavaExprKind::Value(reference) => match reference {
                JavaValueRef::Local(name) => self.name(name),
                JavaValueRef::This
                | JavaValueRef::Generated(_)
                | JavaValueRef::EnumVariant { .. }
                | JavaValueRef::KnownField(_) => {}
            },
            JavaExprKind::Unary { operand, .. } => self.expression(operand),
            JavaExprKind::Binary { left, right, .. } => {
                self.expression(left);
                self.expression(right);
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } => {
                self.expression(condition);
                self.expression(when_true);
                self.expression(when_false);
            }
            JavaExprKind::Call {
                callable,
                receiver,
                arguments,
            } => {
                let signature = match callable {
                    JavaCallableRef::Known { signature, .. }
                    | JavaCallableRef::Runtime { signature, .. }
                    | JavaCallableRef::Generated { signature, .. }
                    | JavaCallableRef::Interface { signature, .. } => signature,
                    JavaCallableRef::Member {
                        owner,
                        name,
                        signature,
                        ..
                    } => {
                        self.ty(owner);
                        self.name(name);
                        signature
                    }
                };
                // Instantiated generic/varargs signatures are not JVM method
                // descriptors. Validate each type, not expanded argument slots.
                self.ty(&signature.result);
                if let Some(ty) = &signature.receiver {
                    self.ty(ty);
                }
                for ty in &signature.parameters {
                    self.ty(ty);
                }
                if let Some(receiver) = receiver {
                    self.expression(receiver);
                }
                for argument in arguments {
                    self.expression(argument);
                }
            }
            JavaExprKind::New {
                constructor,
                arguments,
            } => {
                let parameters = match constructor {
                    JavaConstructorRef::Known {
                        owner, parameters, ..
                    } => {
                        self.ty(owner);
                        parameters
                    }
                    JavaConstructorRef::Generated { parameters, .. } => parameters,
                };
                for ty in parameters {
                    self.ty(ty);
                }
                for value in arguments {
                    self.expression(value);
                }
            }
            JavaExprKind::NewArray { component, length } => {
                self.ty(component);
                self.expression(length);
            }
            JavaExprKind::ArrayIndex { array, index } => {
                self.expression(array);
                self.expression(index);
            }
            JavaExprKind::Field { receiver, field } => {
                self.expression(receiver);
                match field {
                    JavaFieldRef::Known(_) => {}
                    JavaFieldRef::Structural { name, ty }
                    | JavaFieldRef::Generated { name, ty, .. } => {
                        self.name(name);
                        self.ty(ty);
                    }
                }
            }
            JavaExprKind::Cast { target, value }
            | JavaExprKind::InterfaceCoercion { target, value, .. }
            | JavaExprKind::InstanceOf {
                target,
                value,
                binding: None,
            } => {
                self.ty(target);
                self.expression(value);
            }
            JavaExprKind::InstanceOf {
                target,
                value,
                binding: Some(name),
            } => {
                self.ty(target);
                self.expression(value);
                self.name(name);
            }
            JavaExprKind::ArrayOwnershipTransition { value, .. } => self.expression(value),
            JavaExprKind::Lambda { parameters, body } => {
                // Currently rejected by certification. Keep child coverage
                // exhaustive so future admission cannot skip their names/types.
                for parameter in parameters {
                    self.ty(&parameter.ty);
                    self.name(&parameter.name);
                }
                self.block(body);
            }
        }
    }
}
