use crate::{json::Sink, manifest::Manifest, projection::scalar};
use portable_backend_java::{
    ast::JavaDeclaredPath,
    dialect::{JavaSourceDescription, JavaSourceDescriptionKind as Kind, JavaSourceTarget},
};
use portable_codegen::{
    RustDeclarationId, RustExportNamespace, RustExportTarget, RustSourceLocation, RustVisibility,
};

pub(crate) fn owner(out: &mut impl Sink, manifest: &Manifest<'_>) -> Result<(), String> {
    out.fixed("{\"schema_version\":1,\"root\":")?;
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
    out.fixed("]}\n")
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
        Kind::Constant { .. } => {
            return Err("Java constant serialization is not yet admitted".into());
        }
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
            out.string(scalar(result)?)?;
        }
        Kind::Constant { .. } => {
            return Err("Java constant serialization is not yet admitted".into());
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

fn path(out: &mut impl Sink, path: &JavaDeclaredPath) -> Result<(), String> {
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
