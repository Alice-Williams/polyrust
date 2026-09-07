use crate::ast::{JavaKnownType, JavaType};
mod member_names;
pub use member_names::JavaMemberName;
mod known_fields;
pub use known_fields::JavaKnownField;
mod known_callables;
pub use known_callables::JavaKnownCallable;
mod known_constructors;
pub use known_constructors::JavaKnownConstructor;
mod known_methods;
pub use known_methods::JavaKnownMethod;
mod runtime_helpers;
mod signature_matching;
pub use runtime_helpers::{JavaHelperCapability, JavaRuntimeHelper};
mod runtime_callables;
mod signature_builder;
pub use runtime_callables::JavaRuntimeCallable;
mod arena_nodes;
pub use arena_nodes::{JavaArenaExpression, JavaArenaStatement};
mod ast_binding;
mod catalogue;
mod declaration_paths;
mod file_checks;
mod linker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaDialect;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRuntimeType {
    Structural,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaConstructedType(pub JavaType);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaInvocationKind {
    Static,
    Instance,
    Constructor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaSyntheticOrigin {
    Package,
    Test,
    Runtime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaNamespace {
    Type,
    Value,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaNameKey(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPreludeSymbol {
    JavaLang,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaStandardLibrary {
    Jdk21,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaExternalPackage {
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPackageFeature {
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaImportKind {
    Type(JavaKnownType),
}

impl JavaImportKind {
    pub const fn qualified_name(self) -> &'static str {
        match self {
            Self::Type(value) => value.qualified_name(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaQualifiedName {
    Type(JavaKnownType),
    Callable(JavaKnownCallable),
    RuntimeCallable(JavaRuntimeCallable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaGeneratedContainer {
    PublicApi,
}

impl JavaGeneratedContainer {
    pub const fn text(self) -> &'static str {
        match self {
            Self::PublicApi => "Generated",
        }
    }
}

impl JavaQualifiedName {
    pub const fn text(self) -> &'static str {
        match self {
            Self::Type(value) => value.qualified_name(),
            Self::Callable(value) => value.qualified_name(),
            Self::RuntimeCallable(value) => value.qualified_name(),
        }
    }
}

#[cfg(test)]
#[path = "tests/dialect.rs"]
mod tests;
