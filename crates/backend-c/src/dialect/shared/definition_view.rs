//! Read-only manifest input derived from the exact render-ready package.
use super::CDialect;
use crate::ast::{CDefinitionKind, CFileItem, CFileRef, CFunctionRef, CIdentifier, CLinkage};
use portable_codegen::RenderReadyPackage;

/// Members borrowed from actual aggregate declarations in this certificate.
pub fn c_defined_members(
    package: &RenderReadyPackage<CDialect>,
) -> impl Iterator<Item = &crate::ast::CMemberRef> {
    package.ast().files().iter().flat_map(|file| {
        file.items().iter().flat_map(|unit| {
            unit.unit
                .data
                .source
                .items()
                .iter()
                .flat_map(|item| match item {
                    CFileItem::Declaration(declaration) => match declaration.kind() {
                        crate::ast::CDeclarationKind::Aggregate { members, .. } => {
                            members.as_slice()
                        }
                        _ => &[],
                    },
                    _ => &[],
                })
        })
    })
}

/// Conservative checked bound for all generated source/header bytes, before rendering.
pub fn c_output_byte_bound(package: &RenderReadyPackage<CDialect>) -> Result<u64, String> {
    Ok(super::resources::measure_package(package.ast())?
        .total
        .source_bound)
}

/// Cannot manufacture a name/definition pairing independently of certification.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CDefinedFunction;
/// fn forge() { let _ = CDefinedFunction {}; }
/// ```
pub struct CDefinedFunction<'a> {
    function: &'a CFunctionRef,
    name: &'a CIdentifier,
    implementation: &'a CFileRef,
    linkage: CLinkage,
}

impl<'a> CDefinedFunction<'a> {
    pub fn function(&self) -> &'a CFunctionRef {
        self.function
    }
    pub fn name(&self) -> &'a CIdentifier {
        self.name
    }
    pub fn implementation(&self) -> &'a CFileRef {
        self.implementation
    }
    pub fn linkage(&self) -> CLinkage {
        self.linkage
    }
}

pub fn c_defined_functions(
    package: &RenderReadyPackage<CDialect>,
) -> impl Iterator<Item = CDefinedFunction<'_>> {
    package.ast().files().iter().flat_map(|file| {
        file.items().iter().flat_map(|unit| {
            unit.unit
                .data
                .source
                .items()
                .iter()
                .filter_map(move |item| {
                    let CFileItem::Definition(definition) = item else {
                        return None;
                    };
                    let CDefinitionKind::Function {
                        function, linkage, ..
                    } = definition.kind()
                    else {
                        return None;
                    };
                    Some(CDefinedFunction {
                        function,
                        name: &unit.spelling.functions[function],
                        implementation: unit.unit.data.source.identity(),
                        linkage: *linkage,
                    })
                })
        })
    })
}
