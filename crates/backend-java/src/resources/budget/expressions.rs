//! Per-expression bytecode and constant-pool reservations.

use super::Code;
use crate::ast::{JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind, JavaFieldRef};

pub(super) fn expression(value: &JavaExpr) -> Code {
    let mut code = Code {
        bytes: 32,
        pool: 16,
        stack: 8,
        ..Code::default()
    };
    code.ty(&value.ty);
    match &value.kind {
        JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {
            // ldc_w/getstatic are three bytes, a wide local load is four;
            // eight also leaves room for an implicit boxing conversion.
            code.bytes = 8;
        }
        JavaExprKind::Unary { operand, .. } => code.add(&expression(operand)),
        JavaExprKind::Binary { left, right, .. } => {
            code.add(&expression(left));
            code.add(&expression(right));
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            code.add(&expression(condition));
            code.add(&expression(when_true));
            code.add(&expression(when_false));
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
                    owner, signature, ..
                } => {
                    code.ty(owner);
                    signature
                }
            };
            code.ty(&signature.result);
            for ty in &signature.parameters {
                code.ty(ty);
            }
            for ty in &signature.checked_exceptions {
                code.ty(&crate::ast::JavaType::known(*ty));
            }
            if let Some(ty) = &signature.receiver {
                code.ty(ty);
            }
            if let Some(receiver) = receiver {
                code.add(&expression(receiver));
            }
            for argument in arguments {
                code.add(&expression(argument));
                code.argument();
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
                    code.ty(owner);
                    parameters
                }
                JavaConstructorRef::Generated { parameters, .. } => parameters,
            };
            for ty in parameters {
                code.ty(ty);
            }
            for argument in arguments {
                code.add(&expression(argument));
                code.argument();
            }
        }
        JavaExprKind::NewArray { component, length } => {
            code.ty(component);
            code.add(&expression(length));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            code.add(&expression(array));
            code.add(&expression(index));
        }
        JavaExprKind::Field { receiver, field } => {
            code.add(&expression(receiver));
            match field {
                JavaFieldRef::Known(_) => {}
                JavaFieldRef::Structural { ty, .. } | JavaFieldRef::Generated { ty, .. } => {
                    code.ty(ty)
                }
            }
        }
        JavaExprKind::Cast { target, value }
        | JavaExprKind::InterfaceCoercion { target, value, .. } => {
            code.ty(target);
            code.add(&expression(value));
        }
        JavaExprKind::InstanceOf {
            target,
            value,
            binding,
        } => {
            code.ty(target);
            code.add(&expression(value));
            if binding.is_some() {
                code.locals = code.locals.saturating_add(2);
            }
        }
        JavaExprKind::ArrayOwnershipTransition { value, .. } => code.add(&expression(value)),
        JavaExprKind::Lambda { .. } => {
            panic!("uncertified lambda reached Java resource accounting")
        }
    }
    code
}
