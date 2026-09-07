//! Java AST: file model.

use super::declaration_model::{JavaMember, JavaTypeDeclaration};
use portable_codegen::GeneratedSymbolId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaFileItem {
    Type {
        declared: Vec<GeneratedSymbolId>,
        conformances: Box<super::conformance_inventory::JavaConformanceInventory>,
        declaration: JavaTypeDeclaration,
    },
    RuntimeMembers {
        helper: crate::dialect::JavaRuntimeHelper,
        members: Vec<JavaMember>,
    },
}
