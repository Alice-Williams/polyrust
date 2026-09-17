//! Backend-free positive signatures and deliberately ill-typed assignments.
use crate::source_capabilities::{
    ConstantDeclarationInput, ConstantImportInput, ConstantInput, LiteralValue, LocalConstantInput,
    PublicConstantReadInput, ScalarConstantValue,
};

pub(crate) fn distinct(literal: LiteralValue, constant: ScalarConstantValue) {
    let _: LiteralValue = literal;
    let _: ScalarConstantValue = constant;
    #[cfg(constant_domain_literal_to_constant)]
    let _: ScalarConstantValue = literal;
    #[cfg(constant_domain_constant_to_literal)]
    let _: LiteralValue = constant;
}

pub(crate) fn witnesses(
    scalar: ConstantInput<'_>,
    local: LocalConstantInput<'_>,
    declaration: ConstantDeclarationInput<'_>,
    imported: ConstantImportInput<'_>,
    public_read: PublicConstantReadInput<'_>,
) {
    let _: [ScalarConstantValue; 5] = [
        scalar.value(),
        local.value(),
        declaration.value(),
        imported.value(),
        public_read.value(),
    ];
}
