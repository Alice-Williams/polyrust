//! Narrow non-null Result dataflow; general casts and effects stay unadmitted.
use super::Reader;
use crate::{
    ast::{
        JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind, JavaMemberOrigin,
        JavaPrecedence, JavaSynthesizedFieldRole, JavaType, JavaTypeName, JavaValueRef,
    },
    dialect::{JavaKnownCallable, JavaResultTypeRole},
};

impl Reader<'_> {
    /// Return true only after handling an admitted Result operation completely.
    pub(super) fn result_expression(
        &mut self,
        value: &JavaExpr,
        depth: usize,
    ) -> Result<bool, String> {
        match &value.kind {
            JavaExprKind::Value(JavaValueRef::Local(name))
                if self.results.role(&value.ty).is_some() =>
            {
                if !self.nonnull_results.contains(name) {
                    return Err(
                        "Java result local requires explicit non-null boundary handling".into(),
                    );
                }
            }
            JavaExprKind::Call {
                callable:
                    JavaCallableRef::Known {
                        callable: JavaKnownCallable::ObjectsRequireNonNull,
                        signature,
                    },
                receiver: None,
                arguments,
            } if arguments.len() == 1
                && self.results.role(&value.ty).is_some()
                && arguments[0].ty == value.ty
                && value.precedence == JavaPrecedence::Primary =>
            {
                if !JavaKnownCallable::ObjectsRequireNonNull.accepts(signature) {
                    return Err("Java result null check has a substituted signature".into());
                }
                if matches!(
                    arguments[0].kind,
                    JavaExprKind::Value(JavaValueRef::Local(_))
                ) {
                    self.charge(depth + 1)?;
                } else {
                    self.expression(&arguments[0], depth + 1)?;
                }
            }
            JavaExprKind::Cast {
                target,
                value: operand,
            } if self.results.upcast(&operand.ty, target)
                && value.ty == *target
                && value.precedence == JavaPrecedence::Unary =>
            {
                self.expression(operand, depth + 1)?;
            }
            JavaExprKind::InterfaceCoercion {
                target,
                value: operand,
                ..
            } if self.results.upcast(&operand.ty, target)
                && value.ty == *target
                && value.precedence == JavaPrecedence::Primary =>
            {
                self.expression(operand, depth + 1)?;
            }
            JavaExprKind::InstanceOf {
                target,
                value: operand,
                ..
            } if self.results.upcast(target, &operand.ty)
                && value.precedence == JavaPrecedence::Relational =>
            {
                self.expression(operand, depth + 1)?;
            }
            JavaExprKind::New {
                constructor,
                arguments,
            } if self.results.variant(&value.ty) => {
                let expected = match constructor {
                    JavaConstructorRef::Dependency(constructor) => constructor.owner().ty(),
                    JavaConstructorRef::Generated { owner, .. } => {
                        JavaType::Reference(JavaTypeName::Generated(*owner))
                    }
                    _ => {
                        return Err(
                            "Java result construction requires its original constructor".into()
                        );
                    }
                };
                if expected != value.ty || value.precedence != JavaPrecedence::Primary {
                    return Err(
                        "Java result construction has an inconsistent type or precedence".into(),
                    );
                }
                for argument in arguments {
                    self.expression(argument, depth + 1)?;
                }
            }
            JavaExprKind::Call {
                callable: JavaCallableRef::Member { owner, origin, .. },
                receiver: Some(receiver),
                arguments,
            } if self.results.role(owner) == Some(JavaResultTypeRole::Success)
                && receiver.ty == *owner
                && arguments.is_empty()
                && value.precedence == JavaPrecedence::Primary =>
            {
                match origin {
                    JavaMemberOrigin::Dependency(accessor) if accessor.owner().ty() == *owner => {}
                    JavaMemberOrigin::SynthesizedField(field)
                        if field.role == JavaSynthesizedFieldRole::ScalarResultPayload
                            && *owner
                                == JavaType::Reference(JavaTypeName::Generated(field.owner)) => {}
                    _ => {
                        return Err(
                            "Java result observation requires its original payload accessor".into(),
                        );
                    }
                }
                self.expression(receiver, depth + 1)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
