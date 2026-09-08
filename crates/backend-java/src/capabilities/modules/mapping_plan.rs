//! Mapping-owned structural output certificate.
use super::JavaModuleInput;
use crate::ast::{
    JavaDeclarationKind, JavaFileItem, JavaFilePlacement, JavaHeritage, JavaMember, JavaModifier,
    JavaPackage, JavaSourceFileKind, JavaVisibility,
};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
use crate::dialect::JavaDialect;
use crate::lower::{identifier, path, source};
use portable_codegen::{SourceRole, TargetFile};
java_input_plan!(JavaModuleInput, TargetFile<JavaDialect>);
fn representation(_: &JavaModuleInput) -> R {
    R::Declaration
}
fn verify(input: &JavaModuleInput, output: &TargetFile<JavaDialect>) -> bool {
    if output.path() != &path("src/main/java/org/polyrust/generated/Generated.java")
        || output.role() != SourceRole::PublicApi
        || output.module() != &JavaPackage::Generated
        || output.placement() != &JavaFilePlacement::Main
        || output.source_kind() != &JavaSourceFileKind::CompilationUnit
        || output.source() != &source("generated-file")
    {
        return false;
    }
    let [
        JavaFileItem::Type {
            conformances,
            declared,
            declaration: a,
        },
    ] = output.items()
    else {
        return false;
    };
    if declared != &input.declared
        || conformances.as_ref() != &input.conformances
        || a.declared != Some(input.entry)
        || a.kind != JavaDeclarationKind::FinalClass
        || a.visibility != JavaVisibility::Public
        || !a.modifiers.is_empty()
        || a.name != identifier("Generated")
        || !a.type_parameters.is_empty()
        || !a.record_components.is_empty()
        || a.heritage != JavaHeritage::None
        || !a.permits.is_empty()
    {
        return false;
    }
    let Some((JavaMember::Constructor(constructor), members)) = a.members.split_first() else {
        return false;
    };
    constructor.name == identifier("Generated")
        && constructor.modifiers == [JavaModifier::Private]
        && constructor.parameters.is_empty()
        && constructor.body.statements.is_empty()
        && members == input.members
}
