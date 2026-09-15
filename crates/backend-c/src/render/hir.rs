//! Structural grammar formatting, publicly callable only through a certificate.
#[path = "hir/expressions.rs"]
mod expressions;
#[path = "hir/imports.rs"]
mod imports;
#[path = "hir/platform.rs"]
mod platform;
#[path = "hir/statements.rs"]
mod statements;
#[path = "hir/types.rs"]
mod types;

use super::{
    CResolvedUnit, bindings::CValueBinding, documentation::CDocumentationOwner,
    resolved_names::CResolvedNames,
};
use crate::ast::{
    CAggregateRef, CDeclarationKind, CDefinitionKind, CFileItem, CFunctionRef, CLinkage,
    CParameterRef, CReturnType,
};
use portable_codegen::{
    CertifiedSourceFile, LinkedFile, StructuralImportRenderer, TotalSourceRenderer,
};
use std::fmt::Write;

struct Writer<'a> {
    names: &'a CResolvedNames,
}

/// Structural renderer for the certified, closed C profile.
///
/// ```compile_fail
/// use portable_backend_c::dialect::{CDialect, CStructuralRenderer};
/// use portable_codegen::{LinkedFile, TotalSourceRenderer};
/// fn cannot_render_unready(file: &LinkedFile<CDialect>) {
///     CStructuralRenderer.render_file(file);
/// }
/// ```
pub struct CStructuralRenderer;

impl TotalSourceRenderer<super::CDialect> for CStructuralRenderer {
    fn target_name(&self) -> &'static str {
        "c"
    }
    fn render_file(&self, certified: CertifiedSourceFile<'_, super::CDialect>) -> String {
        file(certified.file())
    }
}

pub(super) fn file(file: &LinkedFile<super::CDialect>) -> String {
    let [unit] = file.items() else {
        unreachable!("checked C compilation unit")
    };
    let mut text = String::new();
    if let super::CFileGrammar::Header(guard) = file.source_kind() {
        let name = guard.identifier().as_str();
        writeln!(text, "#ifndef {name}\n#define {name}\n").unwrap();
    }
    text.push_str(&imports::CImports.render_imports(file.imports(), file.file_imports()));
    if !file.imports().is_empty() || !file.file_imports().is_empty() {
        text.push('\n');
    }
    text.push_str(&unit_text(unit));
    if let super::CFileGrammar::Header(guard) = file.source_kind() {
        writeln!(text, "#endif /* {} */", guard.identifier().as_str()).unwrap();
    }
    text
}

fn unit_text(unit: &CResolvedUnit) -> String {
    let writer = Writer {
        names: &unit.spelling,
    };
    let mut text = String::new();
    let documentation = &unit.unit.data.documentation;
    for comment in documentation.modules() {
        writeln!(text, "/* {} */", comment.text()).unwrap();
    }
    for item in unit.unit.data.source.items() {
        match item {
            CFileItem::Comment(comment) => {
                writeln!(text, "/* {} */", comment.text()).unwrap();
            }
            CFileItem::Declaration(declaration) => match declaration.kind() {
                CDeclarationKind::Aggregate {
                    owner: CAggregateRef::Struct(owner),
                    members,
                } => {
                    for comment in
                        documentation.comments(&CDocumentationOwner::Struct(owner.clone()))
                    {
                        writeln!(text, "/* {} */", comment.text()).unwrap();
                    }
                    writeln!(text, "struct {} {{", writer.names.types[owner].as_str()).unwrap();
                    for member in members {
                        for comment in
                            documentation.comments(&CDocumentationOwner::Member(member.clone()))
                        {
                            writeln!(text, "    /* {} */", comment.text()).unwrap();
                        }
                        writeln!(
                            text,
                            "    {};",
                            writer.declarator(
                                member.ty(),
                                writer.names.values[&CValueBinding::Member(member.clone())]
                                    .as_str()
                            )
                        )
                        .unwrap();
                    }
                    text.push_str("};\n");
                }
                CDeclarationKind::FunctionPrototype { function, linkage } => {
                    for comment in
                        documentation.comments(&CDocumentationOwner::Function(function.clone()))
                    {
                        writeln!(text, "/* {} */", comment.text()).unwrap();
                    }
                    writeln!(text, "{};", writer.function(function, *linkage, None)).unwrap();
                }
                CDeclarationKind::ObjectDeclaration(object) => {
                    for comment in
                        documentation.comments(&CDocumentationOwner::Object(object.clone()))
                    {
                        writeln!(text, "/* {} */", comment.text()).unwrap();
                    }
                    writeln!(
                        text,
                        "extern {};",
                        writer.declarator(
                            object.ty(),
                            writer.names.values[&CValueBinding::Global(object.clone())].as_str()
                        )
                    )
                    .unwrap();
                }
                _ => unreachable!("checked declaration profile"),
            },
            CFileItem::Definition(definition) => match definition.kind() {
                CDefinitionKind::Function {
                    function,
                    parameters,
                    body,
                    linkage,
                } => {
                    write!(
                        text,
                        "{} ",
                        writer.function(function, *linkage, Some(parameters))
                    )
                    .unwrap();
                    writer.block(body, 0, &mut text);
                }
                CDefinitionKind::Object {
                    object,
                    linkage: CLinkage::External,
                    initializer,
                } => {
                    writeln!(
                        text,
                        "{} = {};",
                        writer.declarator(
                            object.ty(),
                            writer.names.values[&CValueBinding::Global(object.clone())].as_str()
                        ),
                        writer.initializer(initializer)
                    )
                    .unwrap();
                }
                _ => unreachable!("checked definition profile"),
            },
            CFileItem::StaticAssert(assertion) => writer.platform(assertion, &mut text),
        }
        text.push('\n');
    }
    text
}

impl Writer<'_> {
    fn function(
        &self,
        function: &CFunctionRef,
        linkage: CLinkage,
        parameters: Option<&[CParameterRef]>,
    ) -> String {
        let CReturnType::Value(result) = function.signature().return_type() else {
            unreachable!("checked scalar return")
        };
        let parameters = function
            .signature()
            .parameters()
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                let name = parameters
                    .map(|list| {
                        self.names.values[&CValueBinding::Parameter(list[index].clone())].as_str()
                    })
                    .unwrap_or("");
                let ty = parameters
                    .map(|list| list[index].ty())
                    .unwrap_or(parameter.declared_type());
                self.declarator(ty, name)
            })
            .collect::<Vec<_>>();
        let parameters = if parameters.is_empty() {
            "void".into()
        } else {
            parameters.join(", ")
        };
        let declaration = self.declarator(
            result.declared_type(),
            &format!("{}({parameters})", self.names.functions[function].as_str()),
        );
        match linkage {
            CLinkage::Internal => format!("static {declaration}"),
            CLinkage::External => declaration,
            CLinkage::None => unreachable!("checked linkage profile"),
        }
    }
}
