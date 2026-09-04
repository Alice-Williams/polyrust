//! Java mapping for the complete `Modules` capability.

use portable_build::{CapabilityMapping, Modules};
use portable_codegen::{GeneratedSymbolId, GeneratedTypeId, SourceRole, TargetFile};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaDeclarationKind, JavaFileItem, JavaFilePlacement, JavaHeritage, JavaMember,
        JavaPackage, JavaSourceFileKind, JavaTypeDeclaration, JavaVisibility,
    },
    dialect::JavaDialect,
    lower::{identifier, path, private_constructor, source},
};

#[doc(hidden)]
pub struct JavaModuleInput {
    pub(crate) entry: GeneratedTypeId,
    pub(crate) declared: Vec<GeneratedSymbolId>,
    pub(crate) members: Vec<JavaMember>,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaModules;

impl sealed::JavaCapabilityMapping for JavaModules {}
impl JavaCapabilityMapping for JavaModules {}

impl CapabilityMapping<JavaDialect> for JavaModules {
    type Capability = Modules;
    type Context = ();
    type Input = JavaModuleInput;
    type Output = TargetFile<JavaDialect>;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        mut input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        input
            .members
            .insert(0, JavaMember::Constructor(private_constructor("Generated")));
        let declaration = JavaTypeDeclaration {
            declared: Some(input.entry),
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            modifiers: vec![],
            name: identifier("Generated"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: input.members,
        };
        Ok(TargetFile::new(
            path("src/main/java/org/polyrust/generated/Generated.java"),
            SourceRole::PublicApi,
            JavaPackage::Generated,
            JavaFilePlacement::Main,
            vec![JavaFileItem::Type {
                declared: input.declared,
                declaration,
            }],
            JavaSourceFileKind::CompilationUnit,
            source("generated-file"),
        ))
    }
}
