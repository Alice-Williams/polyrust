//! Java lowering: native test file.

use super::{Lowering, path, source};
use crate::ast::{JavaFileItem, JavaFilePlacement, JavaPackage, JavaSourceFileKind};
use portable_codegen::{SourceRole, TargetFile};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn native_test_file(
        &mut self,
    ) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let declaration = self.test_declaration("GeneratedTest")?;
        Ok(self.builder.file(TargetFile::new(
            path("src/test/java/org/polyrust/generated/GeneratedTest.java"),
            SourceRole::NativeTest,
            JavaPackage::Generated,
            JavaFilePlacement::NativeTest,
            vec![JavaFileItem::Type {
                conformances: crate::ast::JavaConformanceInventory::structural().into(),
                declared: vec![],
                declaration,
            }],
            JavaSourceFileKind::CompilationUnit,
            source("native-test-file"),
        )))
    }
}
