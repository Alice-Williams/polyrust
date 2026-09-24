//! Closed canonical-instance description, never compiler or certificate authority.
use crate::dialect::{JavaErrorKindValues, JavaScalarResultTypes};
use portable_codegen::{RustCanonicalErrorKindFacts, RustCanonicalInstanceKey, TargetPackageOwner};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JavaCanonicalTypeProfile {
    ScalarResultV2,
}

impl JavaCanonicalTypeProfile {
    pub fn namespace(self, instance: RustCanonicalInstanceKey) -> String {
        match self {
            Self::ScalarResultV2 => format!(
                "org.polyrust.generated.t2.c{:016x}.r{:016x}.i32.e{:016x}",
                instance.result_definition().crate_id,
                instance.result_definition().definition_path_hash,
                instance.error_definition().definition_path_hash,
            ),
        }
    }
}

/// Immutable descriptive role bindings. Package verification checks their exact
/// declarations; constructing this value grants no target or compiler witness.
///
/// ```compile_fail
/// use portable_backend_java::ast::{JavaCanonicalTypePackage, JavaCanonicalTypeProfile};
/// fn retarget(package: &mut JavaCanonicalTypePackage) {
///     package.profile = JavaCanonicalTypeProfile::ScalarResultV2;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCanonicalTypePackage {
    profile: JavaCanonicalTypeProfile,
    facts: RustCanonicalErrorKindFacts,
    types: JavaScalarResultTypes,
    kinds: JavaErrorKindValues,
}

impl JavaCanonicalTypePackage {
    pub fn new(
        profile: JavaCanonicalTypeProfile,
        facts: RustCanonicalErrorKindFacts,
        types: JavaScalarResultTypes,
        kinds: JavaErrorKindValues,
    ) -> Self {
        Self {
            profile,
            facts,
            types,
            kinds,
        }
    }
    pub fn profile(&self) -> JavaCanonicalTypeProfile {
        self.profile
    }
    pub fn facts(&self) -> RustCanonicalErrorKindFacts {
        self.facts
    }
    pub fn types(&self) -> JavaScalarResultTypes {
        self.types
    }
    pub fn kinds(&self) -> JavaErrorKindValues {
        self.kinds
    }
    pub fn owner(&self) -> TargetPackageOwner<JavaCanonicalTypeProfile> {
        TargetPackageOwner::CanonicalInstance {
            instance: self.facts.instance().key(),
            profile: self.profile,
        }
    }
}
