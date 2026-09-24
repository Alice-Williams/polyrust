//! Account for the exact structural spellings of admitted Result operations.
use super::Reader;
use crate::{
    ast::{
        JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind, JavaMemberOrigin,
        JavaResolvedName,
    },
    dialect::JavaKnownCallable,
};
use portable_codegen::TargetSymbolRef;

impl Reader<'_> {
    pub(super) fn result_expression(
        &mut self,
        value: &JavaExpr,
        depth: usize,
    ) -> Result<bool, String> {
        match &value.kind {
            JavaExprKind::InterfaceCoercion { target, value, .. } => {
                self.ty(target)?;
                self.expression(value, depth + 1)?;
            }
            JavaExprKind::InstanceOf {
                target,
                value,
                binding,
            } => {
                self.ty(target)?;
                if let Some(name) = binding {
                    self.spelling(name.as_str())?;
                }
                self.expression(value, depth + 1)?;
            }
            JavaExprKind::New {
                constructor: JavaConstructorRef::Dependency(constructor),
                arguments,
            } => {
                self.symbol(TargetSymbolRef::KnownConstructor(
                    constructor.clone().into(),
                ))?;
                for argument in arguments {
                    self.expression(argument, depth + 1)?;
                }
            }
            JavaExprKind::Call {
                callable:
                    JavaCallableRef::Known {
                        callable: JavaKnownCallable::ObjectsRequireNonNull,
                        ..
                    },
                receiver: None,
                arguments,
            } => {
                let callable = JavaKnownCallable::ObjectsRequireNonNull;
                self.symbol(TargetSymbolRef::KnownType(callable.owner().into()))?;
                self.budget.add(1)?;
                self.spelling(callable.name())?;
                for argument in arguments {
                    self.expression(argument, depth + 1)?;
                }
            }
            JavaExprKind::Call {
                callable: JavaCallableRef::Member { origin, name, .. },
                receiver: Some(receiver),
                arguments,
            } if arguments.is_empty() => {
                match origin {
                    JavaMemberOrigin::Dependency(accessor) => {
                        let Some(JavaResolvedName::Member { member, .. }) = self
                            .names
                            .get(&TargetSymbolRef::KnownMethod(accessor.clone().into()))
                        else {
                            return Err("source reservation lacks resolved result accessor".into());
                        };
                        self.spelling(member.text())?;
                    }
                    JavaMemberOrigin::SynthesizedField(_) => self.spelling(name.as_str())?,
                    _ => return Ok(false),
                }
                self.expression(receiver, depth + 1)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
