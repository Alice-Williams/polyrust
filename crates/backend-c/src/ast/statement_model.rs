//! Function-owned statements and explicit lexical/control identities.

use super::{
    CCleanupExitRef, CEffect, CEnumeratorRef, CFunctionRef, CInitializer, CLocalRef, CLoopRef,
    CPlace, CScopeRef, CSignedLiteral, CSwitchRef, CUnsignedLiteral, CValue,
};

/// Statement construction is not flow, initialization or ownership proof.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CStatement, CStatementKind};
/// fn forge(mut statement: CStatement, kind: CStatementKind) { statement.kind = kind; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CStatement {
    pub(super) function: CFunctionRef,
    pub(super) kind: CStatementKind,
}
impl CStatement {
    pub const fn function(&self) -> &CFunctionRef {
        &self.function
    }
    pub const fn kind(&self) -> &CStatementKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CBlock {
    pub(super) scope: CScopeRef,
    pub(super) statements: Vec<CStatement>,
}
impl CBlock {
    pub const fn scope(&self) -> &CScopeRef {
        &self.scope
    }
    pub fn statements(&self) -> &[CStatement] {
        &self.statements
    }
}

/// Automatic local storage is the only admitted category.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CLocalDeclaration, CStorage};
/// fn local_static(mut value: CLocalDeclaration) { value.storage = CStorage::Static; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CLocalDeclaration {
    pub(super) local: CLocalRef,
    pub(super) initializer: Option<CInitializer>,
}
impl CLocalDeclaration {
    pub const fn local(&self) -> &CLocalRef {
        &self.local
    }
    pub const fn initializer(&self) -> Option<&CInitializer> {
        self.initializer.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CStatementKind {
    Empty,
    Block(CBlock),
    Declare(CLocalDeclaration),
    Assign {
        place: CPlace,
        value: CValue,
    },
    Evaluate(CEffect),
    Discard(CValue),
    If {
        condition: CValue,
        then_block: CBlock,
        else_block: CBlock,
    },
    BoundedLoop {
        identity: CLoopRef,
        progress: Box<CCountedProgress>,
        condition: CValue,
        body: CBlock,
    },
    Switch {
        identity: CSwitchRef,
        value: CValue,
        arms: Vec<CSwitchArm>,
        default: CBlock,
    },
    Break(CBreakTarget),
    Continue(CLoopRef),
    Return(Option<CValue>),
    CleanupJump(CCleanupExitRef),
    Label {
        identity: CCleanupExitRef,
        statement: Box<CStatement>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CCountedStep {
    One,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CCountedProgress {
    pub(super) counter: CLocalRef,
    pub(super) bound: CLocalRef,
    pub(super) step: CCountedStep,
}
impl CCountedProgress {
    pub const fn counter(&self) -> &CLocalRef {
        &self.counter
    }
    pub const fn bound(&self) -> &CLocalRef {
        &self.bound
    }
    pub const fn step(&self) -> CCountedStep {
        self.step
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CBreakTarget {
    Loop(CLoopRef),
    Switch(CSwitchRef),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CCaseConstant {
    Signed(CSignedLiteral),
    Unsigned(CUnsignedLiteral),
    Enumerator(CEnumeratorRef),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSwitchArm {
    pub(super) cases: Vec<CCaseConstant>,
    pub(super) body: CBlock,
}
impl CSwitchArm {
    pub fn cases(&self) -> &[CCaseConstant] {
        &self.cases
    }
    pub const fn body(&self) -> &CBlock {
        &self.body
    }
}
