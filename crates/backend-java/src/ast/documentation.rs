//! Resolved documentation has typed owners and non-executable payloads.
use super::JavaDocComment;
use portable_codegen::{GeneratedSymbolId, GeneratedTypeId, RustDeclarationId};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaDocumentationOwner {
    Symbol(GeneratedSymbolId),
    RecordField {
        owner: GeneratedTypeId,
        field: RustDeclarationId,
    },
    Module(RustDeclarationId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaDocumentationStyle {
    Declaration,
    ExplanatoryModule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaDocumentationAttachment {
    pub(crate) indentation: usize,
    pub(crate) style: JavaDocumentationStyle,
    pub(crate) comments: Vec<JavaDocComment>,
}

impl JavaDocumentationAttachment {
    pub fn indentation(&self) -> usize {
        self.indentation
    }
    pub fn style(&self) -> JavaDocumentationStyle {
        self.style
    }
    pub fn comments(&self) -> &[JavaDocComment] {
        &self.comments
    }

    /// Exact fixed-format presentation reservation, including prefixes and LF.
    pub fn presentation_len(&self) -> usize {
        let indentation = self.indentation;
        let opening = match self.style {
            JavaDocumentationStyle::Declaration => "/**\n".len(),
            JavaDocumentationStyle::ExplanatoryModule => "/* Rust module ".len() + 33 + 1,
        };
        let mut bytes = indentation
            .saturating_add(opening)
            .saturating_add(indentation)
            .saturating_add(" */\n".len());
        for line in self
            .comments
            .iter()
            .flat_map(|comment| comment.text().split('\n'))
        {
            bytes = bytes
                .saturating_add(indentation)
                .saturating_add(" * ".len())
                .saturating_add(line.len())
                .saturating_add(1);
        }
        bytes
    }
}

/// Only the Java projection constructs populated collections. An empty collection
/// grants no source, linker or rendering authority.
///
/// ```compile_fail
/// use portable_backend_java::ast::JavaDocumentation;
/// let forged = JavaDocumentation {
///     attachments: Default::default(), module_order: vec![], root: None,
/// };
/// ```
/// Linked items cannot be replaced through a safe public mutable slot:
/// ```
/// use portable_backend_java::{ast::ResolvedJavaFileItem, dialect::JavaDialect};
/// use portable_codegen::LinkedFile;
/// fn inspect(file: &LinkedFile<JavaDialect>) -> &[ResolvedJavaFileItem] {
///     file.items()
/// }
/// ```
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDialect;
/// use portable_codegen::LinkedFile;
/// fn replace(file: &mut LinkedFile<JavaDialect>) {
///     file.items[0].documentation = Default::default();
/// }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JavaDocumentation {
    pub(crate) attachments: BTreeMap<JavaDocumentationOwner, JavaDocumentationAttachment>,
    pub(crate) module_order: Vec<RustDeclarationId>,
    pub(crate) root: Option<RustDeclarationId>,
}

impl JavaDocumentation {
    pub fn get(&self, owner: JavaDocumentationOwner) -> Option<&JavaDocumentationAttachment> {
        self.attachments.get(&owner)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&JavaDocumentationOwner, &JavaDocumentationAttachment)> {
        self.attachments.iter()
    }

    pub fn root(&self) -> Option<RustDeclarationId> {
        self.root
    }
    pub fn modules(&self) -> &[RustDeclarationId] {
        &self.module_order
    }
}
