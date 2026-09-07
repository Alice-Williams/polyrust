//! Java AST: declaration model.

use super::blocks::JavaBlock;
use super::expression_nodes::JavaExpr;
use super::identifiers::JavaIdentifier;
use super::runtime_members::JavaRuntimeMember;
use super::types::{JavaKnownType, JavaType, JavaTypeName};
use portable_codegen::{
    GeneratedCallableId, GeneratedInterfaceMethodId, GeneratedTypeId, GeneratedValueId,
};
use portable_core_ir::CoreFieldId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaVisibility {
    Public,
    Package,
    Private,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaModifier {
    Public,
    Private,
    Static,
    Final,
    Transient,
    Sealed,
    NonSealed,
    Abstract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaAnnotation {
    Override,
    SafeVarargs,
}

impl JavaAnnotation {
    pub const ALL: [Self; 2] = [Self::Override, Self::SafeVarargs];

    pub const fn simple_name(self) -> &'static str {
        match self {
            Self::Override => "Override",
            Self::SafeVarargs => "SafeVarargs",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaParameter {
    pub ty: JavaType,
    pub name: JavaIdentifier,
    pub final_parameter: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaField {
    pub declared: Option<GeneratedValueId>,
    pub modifiers: Vec<JavaModifier>,
    pub ty: JavaType,
    pub name: JavaIdentifier,
    pub initializer: Option<JavaExpr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaEnumConstant {
    pub declared: GeneratedValueId,
    pub name: JavaIdentifier,
}

/// A deliberately ill-typed field used only to prove the native compiler
/// rejects mappings which the portable surface forbids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCompileFailField {
    pub modifiers: Vec<JavaModifier>,
    pub expected_type: JavaType,
    pub name: JavaIdentifier,
    pub initializer: JavaExpr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaMethodDeclaration {
    Structural,
    Callable(GeneratedCallableId),
    Interface(GeneratedInterfaceMethodId),
    Implementation {
        method: portable_core_ir::CoreImplementationMethodId,
        interface: GeneratedInterfaceMethodId,
        witness: super::implementation_witness::JavaImplementationWitness,
    },
    UninhabitedImplementation(GeneratedInterfaceMethodId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaMethod {
    pub declared: JavaMethodDeclaration,
    pub annotations: Vec<JavaAnnotation>,
    pub modifiers: Vec<JavaModifier>,
    pub type_parameters: Vec<JavaIdentifier>,
    pub return_type: JavaType,
    pub name: JavaIdentifier,
    pub parameters: Vec<JavaParameter>,
    pub body: Option<JavaBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaConstructor {
    pub modifiers: Vec<JavaModifier>,
    pub name: JavaIdentifier,
    pub parameters: Vec<JavaParameter>,
    pub body: JavaBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaMember {
    Field(JavaField),
    CompileFailField(JavaCompileFailField),
    EnumConstant(JavaEnumConstant),
    Method(JavaMethod),
    Constructor(JavaConstructor),
    NestedType(JavaTypeDeclaration),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaDeclarationKind {
    FinalClass,
    Record,
    Enum,
    UninhabitedEnum(GeneratedTypeId),
    Interface,
    SealedInterface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaHeritage {
    None,
    Interfaces(Vec<JavaType>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaRecordComponent {
    pub origin: JavaRecordComponentOrigin,
    pub ty: JavaType,
    pub name: JavaIdentifier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRecordComponentOrigin {
    Core(CoreFieldId),
    Runtime(JavaRuntimeMember),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaTypeDeclaration {
    pub declared: Option<GeneratedTypeId>,
    pub kind: JavaDeclarationKind,
    pub visibility: JavaVisibility,
    pub modifiers: Vec<JavaModifier>,
    pub name: JavaIdentifier,
    pub type_parameters: Vec<JavaIdentifier>,
    pub record_components: Vec<JavaRecordComponent>,
    pub heritage: JavaHeritage,
    pub permits: Vec<JavaType>,
    pub members: Vec<JavaMember>,
}

pub(super) fn declaration_owner_type(declaration: &JavaTypeDeclaration) -> Option<JavaType> {
    let raw = if let Some(id) = declaration.declared {
        JavaTypeName::Generated(id)
    } else {
        JavaKnownType::ALL
            .into_iter()
            .find(|known| {
                known.runtime_helper().is_some() && known.simple_name() == declaration.name.as_str()
            })
            .map(JavaTypeName::Known)?
    };
    if declaration.type_parameters.is_empty() {
        Some(JavaType::Reference(raw))
    } else {
        Some(JavaType::Generic {
            raw,
            arguments: declaration
                .type_parameters
                .iter()
                .cloned()
                .map(JavaType::TypeVariable)
                .collect(),
        })
    }
}
