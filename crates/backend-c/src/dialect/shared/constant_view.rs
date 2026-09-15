//! Constant views borrow only the exact certified source and resolved spelling.
use super::{CDialect, bindings::CValueBinding};
use crate::ast::{
    CDefinitionKind, CExpressions, CFileItem, CFileRef, CIdentifier, CInitializerKind, CLinkage,
    CLiteral, CObjectRef, CObjectType, CValueKind,
};
use portable_codegen::RenderReadyPackage;

/// No independent name/object/value pairing can manufacture this view.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CDefinedConstant;
/// fn forge<'a>() -> CDefinedConstant<'a> {
///     CDefinedConstant {
///         object: todo!(), name: todo!(), value: todo!(),
///         implementation: todo!(), linkage: todo!(), read_type: todo!(),
///     }
/// }
/// ```
pub struct CDefinedConstant<'a> {
    object: &'a CObjectRef,
    name: &'a CIdentifier,
    value: &'a CLiteral,
    implementation: &'a CFileRef,
    linkage: CLinkage,
    read_type: CObjectType,
}

impl<'a> CDefinedConstant<'a> {
    pub fn object(&self) -> &'a CObjectRef {
        self.object
    }
    pub fn name(&self) -> &'a CIdentifier {
        self.name
    }
    pub fn value(&self) -> &'a CLiteral {
        self.value
    }
    pub fn implementation(&self) -> &'a CFileRef {
        self.implementation
    }
    pub fn linkage(&self) -> CLinkage {
        self.linkage
    }
    pub fn read_type(&self) -> CObjectType {
        self.read_type.clone()
    }
}

/// Unchecked ASTs cannot supply certified views.
///
/// ```compile_fail
/// use portable_backend_c::dialect::{CDialect, c_defined_constants};
/// use portable_codegen::TargetAstPackage;
/// fn unchecked(package: &TargetAstPackage<CDialect>) {
///     let _ = c_defined_constants(package);
/// }
/// ```
pub fn c_defined_constants(
    package: &RenderReadyPackage<CDialect>,
) -> impl Iterator<Item = CDefinedConstant<'_>> {
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
                    let CDefinitionKind::Object {
                        object,
                        linkage,
                        initializer,
                    } = definition.kind()
                    else {
                        return None;
                    };
                    let CInitializerKind::Expression(value) = initializer.kind() else {
                        unreachable!("certified scalar constant initializer");
                    };
                    let CValueKind::Literal(value) = value.kind() else {
                        unreachable!("certified exact scalar literal");
                    };
                    let expressions =
                        CExpressions::new(unit.unit.projection.registry.registrations());
                    let read = expressions
                        .read(
                            expressions
                                .global(object.clone())
                                .expect("certified registered object"),
                        )
                        .expect("certified scalar read");
                    Some(CDefinedConstant {
                        object,
                        name: &unit.spelling.values[&CValueBinding::Global(object.clone())],
                        value,
                        implementation: unit.unit.data.source.identity(),
                        linkage: *linkage,
                        read_type: read.ty().clone(),
                    })
                })
        })
    })
}
