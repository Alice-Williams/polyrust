//! Rebuild declarations from authoritative inventories and recheck grouping.

use super::super::{
    CDeclaration, CDeclarationKind, CDeclarations, CDefinition, CDefinitionKind, CFileItem,
    CSourceFile,
};
use super::{CContextError as E, Recheck, exact};

impl Recheck<'_> {
    pub(super) fn file(&self, file: &CSourceFile) -> Result<CSourceFile, E> {
        let declarations = CDeclarations::new(self.registry, file.identity().clone())?;
        let items = file
            .items()
            .iter()
            .map(|item| {
                Ok(match item {
                    CFileItem::Declaration(value) => {
                        CFileItem::Declaration(self.declaration(value)?)
                    }
                    CFileItem::Definition(value) => CFileItem::Definition(self.definition(value)?),
                    CFileItem::Comment(value) => CFileItem::Comment(value.clone()),
                    CFileItem::StaticAssert(value) => {
                        let owner = CDeclarations::new(self.registry, value.file().clone())?;
                        CFileItem::StaticAssert(owner.static_assert(
                            self.value(value.condition())?,
                            value.diagnostic().clone(),
                        )?)
                    }
                })
            })
            .collect::<Result<Vec<_>, E>>()?;
        exact(file, declarations.source_file(items)?)
    }

    fn declaration(&self, value: &CDeclaration) -> Result<CDeclaration, E> {
        let ast = CDeclarations::new(self.registry, value.file().clone())?;
        Ok(match value.kind() {
            CDeclarationKind::ForwardTag(owner) => ast.forward_tag(owner.clone())?,
            CDeclarationKind::Typedef(value) => ast.typedef(value.clone())?,
            CDeclarationKind::Aggregate { owner, .. } => ast.aggregate(owner.clone())?,
            CDeclarationKind::Enum { owner, .. } => ast.enumeration(owner.clone())?,
            CDeclarationKind::FunctionPrototype { function, linkage } => {
                ast.function_prototype(function.clone(), *linkage)?
            }
            CDeclarationKind::ObjectDeclaration(value) => ast.object_declaration(value.clone())?,
        })
    }

    fn definition(&self, value: &CDefinition) -> Result<CDefinition, E> {
        let ast = CDeclarations::new(self.registry, value.file().clone())?;
        Ok(match value.kind() {
            CDefinitionKind::Function {
                function,
                linkage,
                parameters,
                body,
            } => ast.function_definition(
                function.clone(),
                *linkage,
                parameters.clone(),
                self.block(function, body)?,
            )?,
            CDefinitionKind::Object {
                object,
                linkage,
                initializer,
            } => ast.object_definition(object.clone(), *linkage, self.initializer(initializer)?)?,
        })
    }
}
