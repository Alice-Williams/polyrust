use std::{fmt, marker::PhantomData};

use portable_ir::v0::NodeId;

pub enum ConstantKind {}
pub enum AliasKind {}
pub enum RecordKind {}
pub enum RecordFieldKind {}
pub enum EnumKind {}
pub enum EnumVariantKind {}
pub enum EnumFieldKind {}
pub enum InterfaceKind {}
pub enum InterfaceMethodKind {}
pub enum ImplementationKind {}
pub enum ImplementationMethodKind {}
pub enum FunctionKind {}
pub enum TestKind {}

/// A stable, declaration-family-specific reference allocated by a module.
pub struct Handle<K> {
    node: NodeId,
    marker: PhantomData<fn() -> K>,
}

impl<K> Handle<K> {
    pub(crate) const fn new(node: NodeId) -> Self {
        Self {
            node,
            marker: PhantomData,
        }
    }

    pub const fn node_id(self) -> NodeId {
        self.node
    }
}

impl<K> Copy for Handle<K> {}

impl<K> Clone for Handle<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> PartialEq for Handle<K> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<K> Eq for Handle<K> {}

impl<K> std::hash::Hash for Handle<K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state);
    }
}

impl<K> fmt::Debug for Handle<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Handle").field(&self.node.0).finish()
    }
}

pub type ConstantId = Handle<ConstantKind>;
pub type AliasId = Handle<AliasKind>;
pub type RecordId = Handle<RecordKind>;
pub type RecordFieldId = Handle<RecordFieldKind>;
pub type EnumId = Handle<EnumKind>;
pub type EnumVariantId = Handle<EnumVariantKind>;
pub type EnumFieldId = Handle<EnumFieldKind>;
pub type InterfaceId = Handle<InterfaceKind>;
pub type InterfaceMethodId = Handle<InterfaceMethodKind>;
pub type ImplementationId = Handle<ImplementationKind>;
pub type ImplementationMethodId = Handle<ImplementationMethodKind>;
pub type FunctionId = Handle<FunctionKind>;
pub type TestId = Handle<TestKind>;

mod sealed {
    pub trait Sealed {}
}

/// Handle families that may appear behind a nominal Core type.
pub trait NamedTypeHandle: sealed::Sealed + Copy {
    fn named_node(self) -> NodeId;
}

macro_rules! named_handle {
    ($kind:ty) => {
        impl sealed::Sealed for Handle<$kind> {}
        impl NamedTypeHandle for Handle<$kind> {
            fn named_node(self) -> NodeId {
                self.node_id()
            }
        }
    };
}

named_handle!(AliasKind);
named_handle!(RecordKind);
named_handle!(EnumKind);
