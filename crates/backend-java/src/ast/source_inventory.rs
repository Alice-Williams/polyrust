//! Linker-derived source registration metadata, not a standalone certificate.
use super::JavaFileItem;
use crate::dialect::JavaDialect;
use portable_codegen::{
    AstViolation, GeneratedCallable, GeneratedInterfaceMethod, GeneratedOrigin, GeneratedSymbolId,
    GeneratedType, GeneratedValue, SynthesisReason, TargetAstPackage,
};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeMap;

/// Typed registration facts retained alongside the resolved declarations.
/// Unsupported source categories remain visible so owner APIs can reject them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum JavaSourceDeclaration {
    Type(GeneratedType<JavaDialect>),
    Callable(GeneratedCallable<JavaDialect>),
    InterfaceMethod(GeneratedInterfaceMethod<JavaDialect>),
    Value(GeneratedValue<JavaDialect>),
}

impl JavaSourceDeclaration {
    pub(crate) fn origin(&self) -> &GeneratedOrigin<JavaDialect> {
        match self {
            Self::Type(value) => &value.origin,
            Self::Callable(value) => &value.origin,
            Self::InterfaceMethod(value) => &value.origin,
            Self::Value(value) => &value.origin,
        }
    }
}

/// Immutable projection of the original registration table for one source item.
/// This registration table is backend-internal, not a public AST construction
/// surface. Its authority comes only from exact rederivation inside a containing
/// render-ready certificate; it does not expose the original unchecked package.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct JavaSourceInventory {
    declarations: BTreeMap<GeneratedSymbolId, JavaSourceDeclaration>,
}

impl JavaSourceInventory {
    pub(crate) fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (&GeneratedSymbolId, &JavaSourceDeclaration)> {
        self.declarations.iter()
    }

    pub(crate) fn get(&self, symbol: GeneratedSymbolId) -> Option<&JavaSourceDeclaration> {
        self.declarations.get(&symbol)
    }

    pub(crate) fn derive(
        package: &TargetAstPackage<JavaDialect>,
        item: &JavaFileItem,
    ) -> Result<Self, AstViolation> {
        let mut declarations = BTreeMap::new();
        for symbol in item.declared_symbols() {
            let value = match symbol {
                GeneratedSymbolId::Type(id) => package
                    .generated_type(id)
                    .map(|value| JavaSourceDeclaration::Type(value.clone())),
                GeneratedSymbolId::Callable(id) => package
                    .callable(id)
                    .map(|value| JavaSourceDeclaration::Callable(value.clone())),
                GeneratedSymbolId::InterfaceMethod(id) => package
                    .interface_method(id)
                    .map(|value| JavaSourceDeclaration::InterfaceMethod(value.clone())),
                GeneratedSymbolId::Value(id) => package
                    .value(id)
                    .map(|value| JavaSourceDeclaration::Value(value.clone())),
            }
            .ok_or_else(|| {
                AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "Java source inventory references an unregistered declaration",
                )
            })?;
            if matches!(value.origin(), GeneratedOrigin::RustSource(_))
                || matches!(&value, JavaSourceDeclaration::Type(value) if matches!(
                    value.origin, GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint)
                ))
            {
                declarations.insert(symbol, value);
            }
        }
        Ok(Self { declarations })
    }
}
