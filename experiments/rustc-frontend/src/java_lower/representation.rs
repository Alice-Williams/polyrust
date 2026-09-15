//! Java-owned representation information survives shared-reference erasure.
use super::Result;
use portable_backend_java::ast::{JavaExpr, JavaPrimitive, JavaType, JavaTypeName};
use portable_codegen::GeneratedTypeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TypePlan {
    I32,
    I64,
    Bool,
    Record(GeneratedTypeId),
    Shared(Box<TypePlan>),
}

impl TypePlan {
    pub(crate) fn java_type(&self) -> JavaType {
        match self {
            Self::I32 => JavaType::primitive(JavaPrimitive::Int),
            Self::I64 => JavaType::primitive(JavaPrimitive::Long),
            Self::Bool => JavaType::primitive(JavaPrimitive::Boolean),
            Self::Record(id) => JavaType::Reference(JavaTypeName::Generated(*id)),
            Self::Shared(referent) => referent.java_type(),
        }
    }

    pub(crate) fn scalar(ty: &JavaType) -> Result<Self> {
        match ty {
            JavaType::Primitive(JavaPrimitive::Int) => Ok(Self::I32),
            JavaType::Primitive(JavaPrimitive::Long) => Ok(Self::I64),
            JavaType::Primitive(JavaPrimitive::Boolean) => Ok(Self::Bool),
            _ => Err("source signature requires an i32/i64/bool representation".into()),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Value {
    plan: TypePlan,
    expression: JavaExpr,
}

impl Value {
    pub(crate) fn new(plan: TypePlan, expression: JavaExpr) -> Result<Self> {
        if expression.ty != plan.java_type() {
            return Err("Java expression disagrees with source representation".into());
        }
        Ok(Self { plan, expression })
    }
    pub(crate) fn plan(&self) -> &TypePlan {
        &self.plan
    }
    pub(crate) fn into_expression(self) -> JavaExpr {
        self.expression
    }
}

/// Only the resolved-place mapping and binding registration construct places.
#[derive(Clone, Debug)]
pub(crate) struct Place(Value);

impl Place {
    pub(super) fn resolved(value: Value) -> Self {
        Self(value)
    }
    pub(crate) fn value(&self) -> Value {
        self.0.clone()
    }
    pub(crate) fn plan(&self) -> &TypePlan {
        self.0.plan()
    }
    pub(crate) fn dereference(self) -> Result<Self> {
        let TypePlan::Shared(referent) = self.0.plan else {
            return Err("built-in dereference requires a shared-reference plan".into());
        };
        Ok(Self(Value::new(*referent, self.0.expression)?))
    }
}
