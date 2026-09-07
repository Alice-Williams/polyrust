//! Reuse the real module/runtime assembly when mutating lowerer-produced members.

use super::{Lowering, path, source};
use crate::ast::{
    JavaConformanceInventory, JavaFileItem, JavaFilePlacement, JavaMember, JavaPackage,
    JavaSourceFileKind, JavaTypeDeclaration,
};
use crate::capabilities::JavaModuleInput;
use portable_build::{CapabilityMapping, Modules};
use portable_codegen::{
    FileGroupRole, SourceRole, TargetFile, TargetFileGroup, TargetFileMember, verify_target_ast,
};
use portable_diagnostics::Diagnostic;

pub(super) fn verify_members(
    lowering: Lowering<'_>,
    members: Vec<JavaMember>,
) -> Result<(), Vec<Diagnostic>> {
    verify_members_with(lowering, members, |_| {})
}

pub(super) fn verify_members_with(
    mut lowering: Lowering<'_>,
    members: Vec<JavaMember>,
    mutate: impl FnOnce(&mut JavaTypeDeclaration),
) -> Result<(), Vec<Diagnostic>> {
    let file = lowering.features.mapping_for::<Modules>().lower(
        &mut (),
        JavaModuleInput {
            conformances: JavaConformanceInventory::from_checked(lowering.core),
            entry: lowering.entry.unwrap(),
            declared: lowering.declared.clone(),
            members,
        },
    )?;
    let mut items = file.items().to_vec();
    let JavaFileItem::Type { declaration, .. } = &mut items[0] else {
        unreachable!()
    };
    mutate(declaration);
    let file = lowering.builder.file(TargetFile::new(
        path("src/main/java/org/polyrust/generated/Generated.java"),
        SourceRole::PublicApi,
        JavaPackage::Generated,
        JavaFilePlacement::Main,
        items,
        JavaSourceFileKind::CompilationUnit,
        source("mutation-module"),
    ));
    let runtime = lowering.runtime_file()?;
    for (role, file) in [
        (FileGroupRole::PublicApi, file),
        (FileGroupRole::Runtime, runtime),
    ] {
        lowering.builder.group(TargetFileGroup::new(
            role,
            vec![TargetFileMember::Source(file)],
            source("mutation-fixture"),
        ));
    }
    verify_target_ast(&lowering.builder.build())
}
