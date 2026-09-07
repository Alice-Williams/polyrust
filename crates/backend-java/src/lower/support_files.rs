//! Java lowering: support files.

use super::declaration_builders::{identifier, private_constructor};
use super::expression_builders::string_literal;
use super::{Lowering, path, source};
use crate::ast::{
    JavaCompileFailField, JavaDeclarationKind, JavaFileItem, JavaFilePlacement, JavaHeritage,
    JavaMember, JavaModifier, JavaPackage, JavaPrimitive, JavaSourceFileKind, JavaType,
    JavaTypeDeclaration, JavaVisibility,
};
use portable_codegen::{SourceRole, TargetFile};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn runtime_file(
        &mut self,
    ) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        Ok(self.builder.file(TargetFile::new(
            path("src/main/java/org/polyrust/generated/Runtime.java"),
            SourceRole::Runtime,
            JavaPackage::Generated,
            JavaFilePlacement::Runtime,
            vec![crate::runtime::shell_item()],
            JavaSourceFileKind::CompilationUnit,
            source("runtime-file"),
        )))
    }

    pub(super) fn conformance_file(
        &mut self,
    ) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let declaration = self.test_declaration("ConformanceTest")?;
        Ok(self.builder.file(TargetFile::new(
            path("src/test/java/org/polyrust/generated/ConformanceTest.java"),
            SourceRole::Conformance,
            JavaPackage::Generated,
            JavaFilePlacement::Conformance,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaSourceFileKind::CompilationUnit,
            source("conformance-file"),
        )))
    }

    pub(super) fn negative_file(
        &mut self,
    ) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let declaration = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: identifier("InvalidTypes"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![
                JavaMember::Constructor(private_constructor("InvalidTypes")),
                JavaMember::CompileFailField(JavaCompileFailField {
                    modifiers: vec![JavaModifier::Final],
                    expected_type: JavaType::primitive(JavaPrimitive::Int),
                    name: identifier("invalid"),
                    initializer: string_literal("missing"),
                }),
            ],
        };
        Ok(self.builder.file(TargetFile::new(
            path("src/test/java/org/polyrust/generated/InvalidTypes.java"),
            SourceRole::NegativeTest,
            JavaPackage::Generated,
            JavaFilePlacement::NegativeTest,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaSourceFileKind::CompilationUnit,
            source("negative-file"),
        )))
    }
}
