use std::collections::{BTreeMap, BTreeSet};

use portable_ir::v0::{Document, Module, NodeId, TypeRef};

/// Stable identity of a resolved parameter, pattern binding, or local binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(NodeId);

impl SymbolId {
    pub(crate) const fn new(node: NodeId) -> Self {
        Self(node)
    }

    pub const fn node_id(self) -> NodeId {
        self.0
    }
}

/// Target-independent semantic features required by a checked program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    Bytes,
    CheckedIntegerArithmetic,
    ContractDispatch,
    F64,
    ImmutableList,
    Option,
    Result,
    UnicodeScalar,
    WrappingIntegerArithmetic,
    BoundedIteration,
}

/// Minimal, deterministic capability sets traceable to requiring IR nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityReport {
    program: BTreeSet<Capability>,
    declarations: BTreeMap<NodeId, BTreeSet<Capability>>,
    nodes: BTreeMap<NodeId, BTreeSet<Capability>>,
}

impl CapabilityReport {
    pub fn program(&self) -> &BTreeSet<Capability> {
        &self.program
    }

    pub fn declaration(&self, declaration: NodeId) -> Option<&BTreeSet<Capability>> {
        self.declarations.get(&declaration)
    }

    pub fn node(&self, node: NodeId) -> Option<&BTreeSet<Capability>> {
        self.nodes.get(&node)
    }

    pub(crate) fn empty() -> Self {
        Self {
            program: BTreeSet::new(),
            declarations: BTreeMap::new(),
            nodes: BTreeMap::new(),
        }
    }

    pub(crate) fn require(&mut self, declaration: NodeId, node: NodeId, capability: Capability) {
        self.program.insert(capability);
        self.declarations
            .entry(declaration)
            .or_default()
            .insert(capability);
        self.nodes.entry(node).or_default().insert(capability);
    }
}

/// Immutable proof that a v0 document passed resolution and semantic checking.
///
/// All fields and its constructor are private to this crate. Safe downstream
/// code can receive or inspect a checked program but cannot forge one.
///
/// ```compile_fail
/// use portable_check::v0::CheckedProgram;
///
/// // Checked programs have no public constructor or public fields.
/// let _forged = CheckedProgram {};
/// ```
#[derive(Clone, Debug)]
pub struct CheckedProgram {
    document: Document,
    expression_types: BTreeMap<NodeId, TypeRef>,
    local_references: BTreeMap<NodeId, SymbolId>,
    capabilities: CapabilityReport,
}

impl CheckedProgram {
    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn module(&self) -> &Module {
        &self.document.module
    }

    pub fn expression_type(&self, expression: NodeId) -> Option<&TypeRef> {
        self.expression_types.get(&expression)
    }

    pub fn expression_types(&self) -> impl Iterator<Item = (NodeId, &TypeRef)> {
        self.expression_types.iter().map(|(node, ty)| (*node, ty))
    }

    pub fn resolved_local(&self, expression: NodeId) -> Option<SymbolId> {
        self.local_references.get(&expression).copied()
    }

    pub fn resolved_locals(&self) -> impl Iterator<Item = (NodeId, SymbolId)> + '_ {
        self.local_references
            .iter()
            .map(|(expression, symbol)| (*expression, *symbol))
    }

    pub fn capabilities(&self) -> &CapabilityReport {
        &self.capabilities
    }

    pub(crate) fn new(
        document: Document,
        expression_types: BTreeMap<NodeId, TypeRef>,
        local_references: BTreeMap<NodeId, SymbolId>,
        capabilities: CapabilityReport,
    ) -> Self {
        Self {
            document,
            expression_types,
            local_references,
            capabilities,
        }
    }
}
