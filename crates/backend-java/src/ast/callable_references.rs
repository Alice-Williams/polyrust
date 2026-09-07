//! Java AST: callable references.

use super::expression_model::{JavaCallableRef, JavaMemberOrigin, JavaMethodSignature};
use super::expression_nodes::{JavaConstructorRef, JavaFieldRef};
use super::generated_members::{
    generated_accessor_matches, generated_constructor_matches, generated_member_matches,
};
use super::types::{JavaType, JavaTypeName};
use crate::dialect::JavaDialect;
use portable_codegen::{TargetAstContext, TargetCallableRef};

impl JavaCallableRef {
    pub(super) fn signature(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> Option<JavaMethodSignature> {
        match self {
            Self::Known {
                callable,
                signature,
            } => callable.accepts(signature).then(|| signature.clone()),
            Self::Runtime {
                callable,
                signature,
            } => callable.accepts(signature).then(|| signature.clone()),
            Self::Generated { symbol, signature } => {
                let registered = context.callable_signature(&TargetCallableRef::Generated(*symbol));
                let actual = JavaDialect.coarse_signature(signature);
                (registered.as_ref() == Some(&actual)
                    && signature.checked_exceptions.is_empty()
                    && !signature.nullable_result
                    && signature.pure)
                    .then(|| signature.clone())
            }
            Self::Interface { symbol, signature } => context
                .callable_signature(&TargetCallableRef::Interface(*symbol))
                .is_some_and(|registered| {
                    registered == JavaDialect.coarse_signature(signature)
                        && signature.checked_exceptions.is_empty()
                        && !signature.nullable_result
                        && signature.pure
                })
                .then(|| signature.clone()),
            Self::Member {
                owner,
                name,
                signature,
                origin,
            } => {
                let owner_matches = signature.receiver.as_ref() == Some(owner);
                let catalogue_matches = match origin {
                    JavaMemberOrigin::Known(method) => method.accepts(signature),
                    JavaMemberOrigin::GeneratedField(field) => {
                        portable_member_metadata_matches(signature)
                            && generated_accessor_matches(owner, *field, name, signature, context)
                    }
                    JavaMemberOrigin::Runtime(member) => {
                        name.as_str() == member.name() && member.accepts(signature)
                    }
                    JavaMemberOrigin::GeneratedImplementation(method) => {
                        portable_member_metadata_matches(signature)
                            && generated_member_matches(
                                owner,
                                name,
                                signature,
                                Some(*method),
                                context,
                            )
                    }
                    JavaMemberOrigin::GeneratedVariant => false,
                };
                (owner_matches && catalogue_matches).then(|| signature.clone())
            }
        }
    }
}

fn portable_member_metadata_matches(signature: &JavaMethodSignature) -> bool {
    signature.checked_exceptions.is_empty() && !signature.nullable_result && signature.pure
}

impl JavaConstructorRef {
    pub(super) fn signature(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> Option<(JavaType, Vec<JavaType>)> {
        match self {
            Self::Known {
                constructor,
                owner,
                parameters,
            } => constructor
                .accepts(owner, parameters)
                .then(|| (owner.clone(), parameters.clone())),
            Self::Generated { owner, parameters }
                if generated_constructor_matches(*owner, parameters, context) =>
            {
                Some((
                    JavaType::Reference(JavaTypeName::Generated(*owner)),
                    parameters.clone(),
                ))
            }
            Self::Generated { .. } => None,
        }
    }
}

impl JavaFieldRef {
    pub(super) fn ty(&self) -> JavaType {
        match self {
            Self::Known(value) => value.ty(),
            Self::Structural { ty, .. } => ty.clone(),
            Self::Generated { ty, .. } => ty.clone(),
        }
    }
}
