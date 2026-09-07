//! Java AST: file model.

use super::declaration_model::{JavaMember, JavaTypeDeclaration};
use portable_codegen::GeneratedSymbolId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaFileItem {
    Type {
        declared: Vec<GeneratedSymbolId>,
        declaration: JavaTypeDeclaration,
    },
    RuntimeMembers {
        helper: crate::dialect::JavaRuntimeHelper,
        members: Vec<JavaMember>,
    },
}
