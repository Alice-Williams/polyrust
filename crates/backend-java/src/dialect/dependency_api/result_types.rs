//! Nominal exports retain one original owner and one certified family identity.
use super::{JavaDependencyApi, JavaDependencyPackage, result_layout::ResultLayout};
use crate::ast::{JavaDeclaredPath, JavaIdentifier, JavaPrimitive};
use portable_codegen::GeneratedTypeId;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaResultTypeRole {
    Interface,
    Success,
    Error,
}

/// Export authority can only be derived by the containing dependency API.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyResultFamily;
/// let forged = JavaDependencyResultFamily {};
/// ```
#[derive(Clone, Debug)]
pub struct JavaDependencyResultFamily {
    owner: JavaDependencyPackage,
    layout: Arc<ResultLayout>,
}

impl JavaDependencyResultFamily {
    pub fn package_identity(&self) -> &JavaDependencyPackage {
        &self.owner
    }
    pub fn ty(&self, role: JavaResultTypeRole) -> JavaDependencyResultType {
        JavaDependencyResultType {
            family: self.clone(),
            role,
        }
    }
    pub(crate) fn local_identity(&self) -> GeneratedTypeId {
        self.layout.types.interface
    }
}
impl PartialEq for JavaDependencyResultFamily {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner && self.local_identity() == other.local_identity()
    }
}
impl Eq for JavaDependencyResultFamily {}
impl PartialOrd for JavaDependencyResultFamily {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for JavaDependencyResultFamily {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.owner, self.local_identity()).cmp(&(&other.owner, other.local_identity()))
    }
}

/// A path is descriptive; holding it cannot construct this nominal witness.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyResultType;
/// let forged = JavaDependencyResultType {};
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaDependencyResultType {
    family: JavaDependencyResultFamily,
    role: JavaResultTypeRole,
}
impl JavaDependencyResultType {
    pub fn family(&self) -> &JavaDependencyResultFamily {
        &self.family
    }
    pub fn role(&self) -> JavaResultTypeRole {
        self.role
    }
    pub fn path(&self) -> &JavaDeclaredPath {
        &self.family.layout.paths[match self.role {
            JavaResultTypeRole::Interface => 0,
            JavaResultTypeRole::Success => 1,
            JavaResultTypeRole::Error => 2,
        }]
    }
    pub fn constructor(&self) -> Option<JavaDependencyResultConstructor> {
        match self.role {
            JavaResultTypeRole::Interface => None,
            JavaResultTypeRole::Success | JavaResultTypeRole::Error => {
                Some(JavaDependencyResultConstructor {
                    owner: self.clone(),
                })
            }
        }
    }
    pub fn payload_accessor(&self) -> Option<JavaDependencyResultAccessor> {
        (self.role == JavaResultTypeRole::Success).then(|| JavaDependencyResultAccessor {
            owner: self.clone(),
        })
    }
}

/// Exact original variant constructor, never the sealed interface constructor.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyResultConstructor;
/// let forged = JavaDependencyResultConstructor {};
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaDependencyResultConstructor {
    owner: JavaDependencyResultType,
}
impl JavaDependencyResultConstructor {
    pub fn owner(&self) -> &JavaDependencyResultType {
        &self.owner
    }
    pub fn parameters(&self) -> &[JavaPrimitive] {
        match self.owner.role {
            JavaResultTypeRole::Success => &[JavaPrimitive::Int],
            JavaResultTypeRole::Error => &[],
            JavaResultTypeRole::Interface => unreachable!("private variant-only constructor"),
        }
    }
}

/// The success record's accessor; this does not grant direct field access.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyResultAccessor;
/// let forged = JavaDependencyResultAccessor {};
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaDependencyResultAccessor {
    owner: JavaDependencyResultType,
}
impl JavaDependencyResultAccessor {
    pub fn owner(&self) -> &JavaDependencyResultType {
        &self.owner
    }
    pub fn name(&self) -> &JavaIdentifier {
        &self.owner.family.layout.payload_name
    }
    pub fn result(&self) -> JavaPrimitive {
        JavaPrimitive::Int
    }
}

impl JavaDependencyApi {
    pub fn result_families(
        &self,
    ) -> impl ExactSizeIterator<Item = JavaDependencyResultFamily> + '_ {
        self.owner
            .0
            .result_families
            .values()
            .map(|layout| family(self.owner.clone(), layout.clone()))
    }
}

pub(super) fn family(
    owner: JavaDependencyPackage,
    layout: Arc<ResultLayout>,
) -> JavaDependencyResultFamily {
    JavaDependencyResultFamily { owner, layout }
}
