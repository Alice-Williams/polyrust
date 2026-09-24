//! Lossless local error-kind family, distinct from the payload-free Result profile.
pub(super) mod check;

use super::{JavaDialect, JavaScalarResultTypes, scalar_result};
use crate::ast::JavaIdentifier;
use portable_codegen::{GeneratedValueId, RenderReadyPackage, RustIntegerErrorKind};
use std::sync::Arc;

/// Descriptive named roles, neither compiler evidence nor target authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JavaErrorKindValues {
    pub empty: GeneratedValueId,
    pub invalid_digit: GeneratedValueId,
    pub positive_overflow: GeneratedValueId,
    pub negative_overflow: GeneratedValueId,
    pub zero: GeneratedValueId,
    pub not_a_power_of_two: GeneratedValueId,
}

impl JavaErrorKindValues {
    pub fn value(self, kind: RustIntegerErrorKind) -> GeneratedValueId {
        match kind {
            RustIntegerErrorKind::Empty => self.empty,
            RustIntegerErrorKind::InvalidDigit => self.invalid_digit,
            RustIntegerErrorKind::PosOverflow => self.positive_overflow,
            RustIntegerErrorKind::NegOverflow => self.negative_overflow,
            RustIntegerErrorKind::Zero => self.zero,
            RustIntegerErrorKind::NotAPowerOfTwo => self.not_a_power_of_two,
        }
    }
}

/// Checked local family only; not source authentication or dependency authority.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaErrorResultFamily;
/// let forged = JavaErrorResultFamily {};
/// ```
#[derive(Clone, Debug)]
pub struct JavaErrorResultFamily {
    package: Arc<RenderReadyPackage<JavaDialect>>,
    types: JavaScalarResultTypes,
    kinds: JavaErrorKindValues,
    payload_name: JavaIdentifier,
}

impl JavaErrorResultFamily {
    pub fn from_certificate(
        package: RenderReadyPackage<JavaDialect>,
        types: JavaScalarResultTypes,
        kinds: JavaErrorKindValues,
    ) -> Result<Self, String> {
        let payload_name = scalar_result::check::checked_family(&package, types, Some(kinds))?;
        Ok(Self {
            package: Arc::new(package),
            types,
            kinds,
            payload_name,
        })
    }
    pub fn package(&self) -> &RenderReadyPackage<JavaDialect> {
        &self.package
    }
    pub fn types(&self) -> JavaScalarResultTypes {
        self.types
    }
    pub fn error_value(&self, kind: RustIntegerErrorKind) -> GeneratedValueId {
        self.kinds.value(kind)
    }
    pub fn payload_name(&self) -> &JavaIdentifier {
        &self.payload_name
    }
}
