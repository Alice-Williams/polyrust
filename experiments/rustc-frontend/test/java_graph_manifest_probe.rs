//! Independent typed observations, never the production JSON projection/encoder.
use super::hex;
use portable_backend_java::{
    ast::{JavaDeclaredPath, JavaPrimitive, JavaType},
    dialect::{JavaDependencyApi, JavaSourceDescriptionKind as Kind, JavaSourceTarget},
};
use portable_codegen::{RustDeclarationId, RustExportNamespace, RustExportTarget, RustVisibility};
use std::{collections::BTreeMap, fmt::Write};

fn id(value: RustDeclarationId) -> String {
    format!(
        "{:016x}:{:016x}",
        value.crate_id, value.definition_path_hash
    )
}
fn path(value: &JavaDeclaredPath) -> String {
    std::iter::once(value.package().name().into_owned())
        .chain(value.owners().iter().map(|n| n.as_str().to_owned()))
        .chain(std::iter::once(value.member().as_str().to_owned()))
        .collect::<Vec<_>>()
        .join(".")
}
fn scalar(value: &JavaType) -> &'static str {
    match value {
        JavaType::Primitive(JavaPrimitive::Int) => "i32",
        JavaType::Primitive(JavaPrimitive::Boolean) => "bool",
        _ => panic!("source scalar"),
    }
}

pub(super) fn write(api: &JavaDependencyApi, output: &mut String) {
    let descriptions = api.source_descriptions().unwrap();
    let exports = &descriptions[0].source().crate_exports;
    let mut modules = BTreeMap::new();
    for chain in exports
        .module_ancestries
        .values()
        .chain(descriptions.iter().map(|d| &d.source().module_ancestors))
    {
        for module in chain.iter() {
            if let Some(previous) = modules.insert(module.declaration, module) {
                assert_eq!(previous, module);
            }
        }
    }
    for description in descriptions {
        let source = description.source();
        let (kind, first, second) = match description.kind() {
            Kind::Function { parameters, result } => (
                "function",
                parameters
                    .iter()
                    .map(|p| scalar(&p.ty))
                    .collect::<Vec<_>>()
                    .join(","),
                scalar(result).into(),
            ),
            Kind::Record => ("record", "-".into(), "-".into()),
            Kind::Field { owner, ty } => ("field", scalar(ty).into(), id(owner)),
        };
        let target = match description.target() {
            JavaSourceTarget::Declaration(value) => path(value),
            JavaSourceTarget::Field { owner, member } => {
                format!("{}.{}", path(owner), member.as_str())
            }
        };
        let (target_kind, target_path, field) = match description.target() {
            JavaSourceTarget::Declaration(value) => ("declaration", value, "-"),
            JavaSourceTarget::Field { owner, member } => ("field", owner, member.as_str()),
        };
        writeln!(
            output,
            "TARGET\t{}\t{target_kind}\t{}\t{}\t{}\t{field}",
            id(source.declaration),
            hex(&target_path.package().name()),
            target_path
                .owners()
                .iter()
                .map(|n| n.as_str())
                .collect::<Vec<_>>()
                .join(","),
            target_path.member().as_str()
        )
        .unwrap();
        let visibility = match source.visibility {
            RustVisibility::Public => "public".into(),
            RustVisibility::RestrictedTo(module) => id(module),
        };
        writeln!(
            output,
            "SOURCE\t{}\t{kind}\t{}\t{visibility}\t{}\t{}\t{}\t{}\t{target}\t{first}\t{second}",
            id(source.declaration),
            id(source.module),
            source.externally_reachable,
            hex(&source.location.file),
            source.location.line,
            source.location.column
        )
        .unwrap();
        for doc in &source.documentation {
            writeln!(
                output,
                "SOURCEDOC\t{}\t{}",
                id(source.declaration),
                hex(doc)
            )
            .unwrap();
        }
    }
    for (identity, module) in modules {
        writeln!(
            output,
            "MODULE\t{}\t{}\t{}\t{}\t{}",
            id(identity),
            module.parent.map(id).unwrap_or_else(|| "-".into()),
            hex(&module.location.file),
            module.location.line,
            module.location.column
        )
        .unwrap();
        for doc in &module.documentation {
            writeln!(output, "MODULEDOC\t{}\t{}", id(identity), hex(doc)).unwrap();
        }
        if let Some(bindings) = exports.modules.get(&identity) {
            for (name, target) in bindings {
                let namespace = match name.namespace {
                    RustExportNamespace::Type => "type",
                    RustExportNamespace::Value => "value",
                    RustExportNamespace::Macro => panic!("macro"),
                };
                let (kind, target) = match target {
                    RustExportTarget::Module(target) => ("module", target),
                    RustExportTarget::Declaration(target) => ("declaration", target),
                };
                writeln!(
                    output,
                    "BINDING\t{}\t{namespace}\t{}\t{kind}\t{}",
                    id(identity),
                    hex(&name.name),
                    id(*target)
                )
                .unwrap();
            }
        }
    }
}
