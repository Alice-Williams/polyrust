//! Exact callable origins and distinct value/effect calls.

use super::{CCallableContractRef, CFunctionRef, CFunctionType, CValue, registry::RegistryScope};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CCallable {
    pub(super) brand: RegistryScope,
    pub(super) kind: CCallableKind,
    pub(super) function: CFunctionRef,
}

impl CCallable {
    pub const fn kind(&self) -> &CCallableKind {
        &self.kind
    }
    pub fn signature(&self) -> &CFunctionType {
        self.function.signature()
    }
    pub fn contract(&self) -> &CCallableContractRef {
        self.function.contract()
    }
    pub const fn contract_function(&self) -> &CFunctionRef {
        &self.function
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CCallableKind {
    Direct,
    /// The verifier must prove this expression's actual callable provenance.
    Indirect(Box<CValue>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CCall {
    pub(super) callable: CCallable,
    pub(super) arguments: Vec<CValue>,
}

impl CCall {
    pub const fn callable(&self) -> &CCallable {
        &self.callable
    }
    pub fn arguments(&self) -> &[CValue] {
        &self.arguments
    }
}

/// A void call cannot be passed as a value, argument or initializer.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CExpressions, CEffect};
/// fn invalid(ast: &CExpressions<'_>, effect: CEffect) {
///     ast.numeric_conversion(portable_backend_c::ast::CScalarType::Bool, effect);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CEffect {
    pub(super) call: CCall,
}

impl CEffect {
    pub const fn call(&self) -> &CCall {
        &self.call
    }
}
