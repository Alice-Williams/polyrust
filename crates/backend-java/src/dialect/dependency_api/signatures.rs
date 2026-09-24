//! Producer-local indices never cross the exported signature boundary.
use super::{JavaDependencyPackage, JavaDependencyResultType};
use crate::ast::{JavaMethodSignature, JavaPrimitive, JavaType, JavaTypeName};

/// An exported type uses original certificate identity, never an arena index.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaDependencyType {
    Primitive(JavaPrimitive),
    Result(JavaDependencyResultType),
}

/// Immutable projection of a verified static function's signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaDependencySignature {
    parameters: Vec<JavaDependencyType>,
    result: JavaDependencyType,
}

impl JavaDependencySignature {
    pub fn parameters(&self) -> &[JavaDependencyType] {
        &self.parameters
    }
    pub fn result(&self) -> &JavaDependencyType {
        &self.result
    }
    pub(super) fn project(
        owner: &JavaDependencyPackage,
        signature: &JavaMethodSignature,
    ) -> Result<Self, String> {
        if signature.receiver.is_some()
            || !signature.checked_exceptions.is_empty()
            || signature.nullable_result
            || !signature.pure
        {
            return Err("Java exported function has unadmitted signature metadata".into());
        }
        Ok(Self {
            parameters: signature
                .parameters
                .iter()
                .map(|ty| project_type(owner, ty, false))
                .collect::<Result<_, _>>()?,
            result: project_type(owner, &signature.result, true)?,
        })
    }
}

fn project_type(
    owner: &JavaDependencyPackage,
    ty: &JavaType,
    result: bool,
) -> Result<JavaDependencyType, String> {
    match ty {
        JavaType::Primitive(value)
            if matches!(
                value,
                JavaPrimitive::Int
                    | JavaPrimitive::Long
                    | JavaPrimitive::Boolean
                    | JavaPrimitive::Double
            ) || (result && *value == JavaPrimitive::Void) =>
        {
            Ok(JavaDependencyType::Primitive(*value))
        }
        JavaType::Reference(JavaTypeName::Imported(value)) => {
            Ok(JavaDependencyType::Result(value.original().clone()))
        }
        JavaType::Reference(JavaTypeName::Generated(id)) => {
            let (layout, role) = owner
                .0
                .result_types
                .get(id)
                .ok_or("Java exported signature references an unselected local nominal")?;
            Ok(JavaDependencyType::Result(
                super::result_types::family(owner.clone(), layout.clone()).ty(*role),
            ))
        }
        _ => Err("Java exported signature contains an unadmitted type".into()),
    }
}
