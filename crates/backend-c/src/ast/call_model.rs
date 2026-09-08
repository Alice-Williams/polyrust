//! Exact callable origins and distinct value/effect calls.

use std::borrow::Cow;

use super::{CFunctionRef, CFunctionType, CValue, registry::RegistryScope};
use crate::dialect::{CKnownCall, CKnownOperands};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CCallable {
    pub(super) brand: RegistryScope,
    pub(super) kind: CCallableKind,
}

impl CCallable {
    pub const fn kind(&self) -> &CCallableKind {
        &self.kind
    }
    pub fn signature(&self) -> Cow<'_, CFunctionType> {
        match &self.kind {
            CCallableKind::Direct(function)
            | CCallableKind::Indirect {
                contract_function: function,
                ..
            } => Cow::Borrowed(function.signature()),
            CCallableKind::Known(call) => Cow::Owned(call.signature()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CCallableKind {
    Direct(Box<CFunctionRef>),
    /// The verifier must prove this expression's actual callable provenance.
    Indirect {
        pointer: Box<CValue>,
        contract_function: Box<CFunctionRef>,
    },
    Known(CKnownCall),
}

/// A body's generated contract and a standard contract are distinct categories.
/// Neither this projection nor a matching signature proves a body's effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CCallContract<'a> {
    Generated {
        function: &'a CFunctionRef,
        arguments: &'a [CValue],
    },
    Known(CKnownOperands<'a>),
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

    pub fn contract(&self) -> CCallContract<'_> {
        match self.callable.kind() {
            CCallableKind::Direct(function)
            | CCallableKind::Indirect {
                contract_function: function,
                ..
            } => CCallContract::Generated {
                function,
                arguments: self.arguments(),
            },
            CCallableKind::Known(call) => CCallContract::Known(call.bind(self.arguments())),
        }
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
