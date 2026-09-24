//! A bounded local Result representation is narrower than valid Java syntax.
pub(super) mod check;

use super::JavaDialect;
use crate::ast::{JavaIdentifier, JavaSynthesizedField, JavaSynthesizedFieldRole};
use portable_codegen::{GeneratedTypeId, RenderReadyPackage};
use std::sync::Arc;

/// Descriptive selection, authenticated against the exact immutable certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JavaScalarResultTypes {
    pub interface: GeneratedTypeId,
    pub success: GeneratedTypeId,
    pub error: GeneratedTypeId,
}

/// Proof of this local declaration family, not Rust provenance or import authority.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaScalarResultFamily;
/// let forged = JavaScalarResultFamily {};
/// ```
#[derive(Clone, Debug)]
pub struct JavaScalarResultFamily {
    package: Arc<RenderReadyPackage<JavaDialect>>,
    types: JavaScalarResultTypes,
    payload_name: JavaIdentifier,
}

impl JavaScalarResultFamily {
    pub fn from_certificate(
        package: RenderReadyPackage<JavaDialect>,
        types: JavaScalarResultTypes,
    ) -> Result<Self, String> {
        let payload_name = Self::checked_payload_name(&package, types)?;
        Ok(Self {
            package: Arc::new(package),
            types,
            payload_name,
        })
    }
    pub fn package(&self) -> &RenderReadyPackage<JavaDialect> {
        &self.package
    }
    pub(in crate::dialect) fn checked_payload_name(
        package: &RenderReadyPackage<JavaDialect>,
        types: JavaScalarResultTypes,
    ) -> Result<JavaIdentifier, String> {
        check::family(package, types)
    }
    pub fn types(&self) -> JavaScalarResultTypes {
        self.types
    }
    pub fn payload(&self) -> JavaSynthesizedField {
        JavaSynthesizedField {
            owner: self.types.success,
            role: JavaSynthesizedFieldRole::ScalarResultPayload,
        }
    }
    pub fn payload_name(&self) -> &JavaIdentifier {
        &self.payload_name
    }
}
