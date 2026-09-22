use crate::{
    json::Sink,
    manifest::Manifest,
    projection::{function_result, scalar},
};
use portable_backend_java::{
    ast::{JavaDeclaredPath, JavaPrimitive, JavaType},
    dialect::{JavaSourceDescription, JavaSourceDescriptionKind as Kind, JavaSourceTarget},
};
use portable_codegen::{
    RustDeclarationId, RustExportNamespace, RustExportTarget, RustSourceLocation, RustVisibility,
};

pub(crate) fn owner(out: &mut impl Sink, manifest: &Manifest<'_>) -> Result<(), String> {
    let imports = crate::constant_imports::collect(manifest.owner.api)?;
    let unit_results = manifest.declarations.iter().any(|description| {
        matches!(description.kind(), Kind::Function { result, .. }
            if *result == portable_backend_java::ast::JavaType::primitive(portable_backend_java::ast::JavaPrimitive::Void))
    });
    let is_f64 = |ty: &JavaType| matches!(ty, JavaType::Primitive(JavaPrimitive::Double));
    let binary64 = manifest
        .declarations
        .iter()
        .any(|description| match description.kind() {
            Kind::Function { parameters, result } => {
                is_f64(result) || parameters.iter().any(|parameter| is_f64(&parameter.ty))
            }
            Kind::Field { ty, .. } | Kind::Constant { ty, .. } => is_f64(ty),
            Kind::Record => false,
        })
        || imports.values().any(|proof| is_f64(proof.ty()))
        || manifest
            .foreign_constants
            .iter()
            .any(|binding| is_f64(binding.dependency().ty()));
    out.fixed(
        if manifest
            .owner
            .api
            .source_types()
            .is_some_and(portable_codegen::RustSourceTypes::contains_char)
        {
            "{\"schema_version\":7,\"root\":"
        } else if binary64 {
            "{\"schema_version\":6,\"root\":"
        } else if unit_results {
            "{\"schema_version\":5,\"root\":"
        } else if !manifest.foreign_constants.is_empty() {
            "{\"schema_version\":4,\"root\":"
        } else if !imports.is_empty() {
            "{\"schema_version\":3,\"root\":"
        } else if manifest.owner.api.constants().len() == 0 {
            "{\"schema_version\":1,\"root\":"
        } else {
            "{\"schema_version\":2,\"root\":"
        },
    )?;
    out.id(manifest.owner.api.root())?;
    out.fixed(",\"defining_key\":")?;
    out.string(manifest.owner.key)?;
    out.fixed(",\"source\":")?;
    out.string(&manifest.source)?;
    out.fixed(",\"modules\":[")?;
    for (position, (id, module)) in manifest.modules.iter().enumerate() {
        comma(out, position)?;
        out.fixed("{\"id\":")?;
        out.id(*id)?;
        out.fixed(",\"parent\":")?;
        if let Some(parent) = module.parent {
            out.id(parent)?;
        } else {
            out.fixed("null")?;
        }
        out.fixed(",\"location\":")?;
        location(out, &module.location)?;
        out.fixed(",\"documentation\":")?;
        docs(out, &module.documentation)?;
        out.fixed(",\"bindings\":[")?;
        if let Some(bindings) = manifest.exports.modules.get(id) {
            for (position, (name, target)) in bindings.iter().enumerate() {
                comma(out, position)?;
                out.fixed("{\"namespace\":")?;
                out.string(match name.namespace {
                    RustExportNamespace::Type => "type",
                    RustExportNamespace::Value => "value",
                    RustExportNamespace::Macro => return Err("unsupported macro binding".into()),
                })?;
                out.fixed(",\"name\":")?;
                out.string(&name.name)?;
                out.fixed(",\"target\":{\"kind\":")?;
                let (kind, id) = match target {
                    RustExportTarget::Module(id) => ("module", id),
                    RustExportTarget::Declaration(id) => ("declaration", id),
                };
                out.string(kind)?;
                out.fixed(",\"id\":")?;
                out.id(*id)?;
                out.fixed("}}")?;
            }
        }
        out.fixed("]}")?;
    }
    out.fixed("],\"declarations\":[")?;
    for (position, description) in manifest.declarations.iter().enumerate() {
        comma(out, position)?;
        declaration(out, description)?;
    }
    out.fixed("],\"dependencies\":[")?;
    for (position, dependency) in manifest.owner.api.dependencies().enumerate() {
        comma(out, position)?;
        out.id(dependency.root())?;
    }
    out.fixed("]")?;
    if !imports.is_empty() {
        crate::constant_imports::write(out, &imports)?;
    }
    if !manifest.foreign_constants.is_empty() {
        crate::constant_exports::write(out, &manifest.foreign_constants)?;
    }
    crate::source_types::write(out, manifest.owner.api.source_types())?;
    out.fixed("}\n")
}

pub(crate) fn index(
    out: &mut impl Sink,
    root: RustDeclarationId,
    members: &[Manifest<'_>],
) -> Result<(), String> {
    out.fixed("{\"schema_version\":1,\"target\":\"org.polyrust.java\",\"root\":")?;
    out.id(root)?;
    out.fixed(",\"members\":[")?;
    for (position, member) in members.iter().enumerate() {
        comma(out, position)?;
        out.fixed("{\"root\":")?;
        out.id(member.owner.api.root())?;
        out.fixed(",\"source\":")?;
        out.string(&member.source)?;
        out.fixed(",\"manifest\":")?;
        out.string(&member.filename)?;
        out.fixed("}")?;
    }
    out.fixed("]}\n")
}

fn declaration(out: &mut impl Sink, description: &JavaSourceDescription<'_>) -> Result<(), String> {
    let source = description.source();
    out.fixed("{\"id\":")?;
    out.id(source.declaration)?;
    out.fixed(",\"kind\":")?;
    out.string(match description.kind() {
        Kind::Function { .. } => "function",
        Kind::Constant { .. } => "constant",
        Kind::Record => "record",
        Kind::Field { .. } => "field",
    })?;
    out.fixed(",\"module\":")?;
    out.id(source.module)?;
    out.fixed(",\"location\":")?;
    location(out, &source.location)?;
    out.fixed(",\"visibility\":{\"kind\":")?;
    match source.visibility {
        RustVisibility::Public => out.string("public")?,
        RustVisibility::RestrictedTo(id) => {
            out.string("restricted_to")?;
            out.fixed(",\"module\":")?;
            out.id(id)?;
        }
    }
    out.fixed("},\"externally_reachable\":")?;
    out.fixed(if source.externally_reachable {
        "true"
    } else {
        "false"
    })?;
    out.fixed(",\"documentation\":")?;
    docs(out, &source.documentation)?;
    out.fixed(",\"target\":")?;
    match description.target() {
        JavaSourceTarget::Declaration(target) => {
            out.fixed("{\"kind\":\"declaration\",\"path\":")?;
            path(out, target)?;
        }
        JavaSourceTarget::Field { owner, member } => {
            out.fixed("{\"kind\":\"field\",\"owner\":")?;
            path(out, owner)?;
            out.fixed(",\"member\":")?;
            out.string(member.as_str())?;
        }
    }
    out.fixed("}")?;
    match description.kind() {
        Kind::Function { parameters, result } => {
            out.fixed(",\"parameters\":[")?;
            for (position, parameter) in parameters.iter().enumerate() {
                comma(out, position)?;
                out.string(scalar(&parameter.ty)?)?;
            }
            out.fixed("],\"result\":")?;
            out.string(function_result(result)?)?;
        }
        Kind::Constant { ty, value } => {
            out.fixed(",\"scalar\":")?;
            out.string(scalar(ty)?)?;
            out.fixed(",\"readonly\":true,\"value\":")?;
            crate::constants::certified_value(out, value)?;
        }
        Kind::Record => {}
        Kind::Field { owner, ty } => {
            out.fixed(",\"owner\":")?;
            out.id(owner)?;
            out.fixed(",\"scalar\":")?;
            out.string(scalar(ty)?)?;
        }
    }
    out.fixed("}")
}

pub(crate) fn path(out: &mut impl Sink, path: &JavaDeclaredPath) -> Result<(), String> {
    out.fixed("{\"package\":")?;
    out.string(&path.package().name())?;
    out.fixed(",\"owners\":[")?;
    for (position, owner) in path.owners().iter().enumerate() {
        comma(out, position)?;
        out.string(owner.as_str())?;
    }
    out.fixed("],\"member\":")?;
    out.string(path.member().as_str())?;
    out.fixed("}")
}
fn location(out: &mut impl Sink, location: &RustSourceLocation) -> Result<(), String> {
    out.fixed("{\"file\":")?;
    out.string(&location.file)?;
    out.fixed(",\"line\":")?;
    out.number(location.line)?;
    out.fixed(",\"column\":")?;
    out.number(location.column)?;
    out.fixed("}")
}
fn docs(out: &mut impl Sink, docs: &[String]) -> Result<(), String> {
    out.fixed("[")?;
    for (position, doc) in docs.iter().enumerate() {
        comma(out, position)?;
        out.string(doc)?;
    }
    out.fixed("]")
}
fn comma(out: &mut impl Sink, position: usize) -> Result<(), String> {
    if position != 0 {
        out.fixed(",")?;
    }
    Ok(())
}
