//! Typed Java syntax models and responsibility-specific verification.

mod blocks;
pub use blocks::{JavaBlock, JavaLocalFinality};
mod callable_references;
mod completion;
mod constructor_flow;
mod declaration_grammar;
mod declaration_model;
pub use declaration_model::{
    JavaAnnotation, JavaCompileFailField, JavaConstructor, JavaDeclarationKind, JavaEnumConstant,
    JavaField, JavaHeritage, JavaMember, JavaMethod, JavaMethodDeclaration, JavaModifier,
    JavaParameter, JavaRecordComponent, JavaRecordComponentOrigin, JavaTypeDeclaration,
    JavaVisibility,
};
mod declarations;
mod exceptions;
mod expression_arena;
mod expression_model;
pub use expression_model::{
    JavaBinaryOperator, JavaCallableRef, JavaLiteral, JavaMemberOrigin, JavaMethodSignature,
    JavaNullPurpose, JavaPrecedence, JavaUnaryOperator, JavaValueRef,
};
mod expression_nodes;
pub use expression_nodes::{JavaConstructorRef, JavaExpr, JavaExprKind, JavaFieldRef};
mod expressions;
mod field_metadata;
mod file_model;
pub use file_model::JavaFileItem;
mod file_symbols;
mod file_verification;
mod final_assignment;
mod generated_members;
mod identifiers;
pub use identifiers::JavaIdentifier;
mod initializers;
mod interface_conformance;
mod interface_witness;
pub use interface_witness::JavaInterfaceWitness;
mod invocations;
mod lexical_blocks;
mod lexical_expressions;
mod lexical_scope;
mod members;
mod method_contracts;
mod modifiers;
mod object_members;
mod operator_signatures;
mod privileged_literals;
mod resolved_files;
pub use resolved_files::{
    JavaCompilationUnit, JavaFilePlacement, JavaPackage, JavaResolvedName, JavaSourceFileKind,
    ResolvedJavaFileItem,
};
mod runtime_members;
pub use runtime_members::JavaRuntimeMember;
mod sealed_permits;
mod statement_arena;
mod statement_context;
mod statement_model;
pub use statement_model::{JavaCatch, JavaPattern, JavaStmt, JavaSwitchArm};
mod statements;
mod switch_patterns;
mod type_context;
mod types;
pub use types::{
    JavaArrayOwnership, JavaArrayOwnershipTransition, JavaKnownType, JavaPrimitive, JavaType,
    JavaTypeName, JavaTypeUse, JavaWildcardBound,
};

#[cfg(test)]
#[path = "tests/ast.rs"]
mod tests;
