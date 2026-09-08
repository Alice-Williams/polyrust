//! File items must belong to their authenticated grouping.

use super::constant_expressions::is_integer_constant_expression;
use super::{
    CAssertDiagnostic, CDeclarations, CExpressions, CFileError as E, CFileItem, CSourceFile,
    CStaticAssertion, CValue,
};

impl CDeclarations<'_> {
    pub fn source_file(&self, items: Vec<CFileItem>) -> Result<CSourceFile, E> {
        for item in &items {
            match item {
                CFileItem::Declaration(value) => self.same_file(value.file())?,
                CFileItem::Definition(value) => self.same_file(value.file())?,
                CFileItem::StaticAssert(value) => self.same_file(value.file())?,
                CFileItem::Comment(_) => {}
            }
        }
        Ok(CSourceFile {
            file: self.file.clone(),
            items,
        })
    }

    /// Checks the constant-expression category, not assertion truth. The actual
    /// tree and message remain available for independent contextual checking.
    pub fn static_assert(
        &self,
        condition: CValue,
        diagnostic: CAssertDiagnostic,
    ) -> Result<CStaticAssertion, E> {
        CExpressions::new(self.registry).check_value(&condition)?;
        if !is_integer_constant_expression(&condition) {
            return Err(E::ExpectedIntegerConstantExpression);
        }
        Ok(CStaticAssertion {
            file: self.file.clone(),
            condition,
            diagnostic,
        })
    }
}
