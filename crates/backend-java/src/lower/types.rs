//! Java lowering: types.

use super::{Lowering, diagnostic};
use crate::ast::{JavaKnownType, JavaType};
use crate::capabilities::{
    JavaBoolValuesInput, JavaBytesInput, JavaCharValuesInput, JavaEnumsInput, JavaEnumsNode,
    JavaF64ValuesInput, JavaI32ValuesInput, JavaI64ValuesInput, JavaInterfacesInput,
    JavaInterfacesNode, JavaListInput, JavaOptionInput, JavaRecordsInput, JavaRecordsNode,
    JavaResultInput, JavaTextValuesInput, JavaUnitValuesInput,
};
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreType, CoreTypeId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn ty(&self, id: CoreTypeId) -> Result<JavaType, Vec<Diagnostic>> {
        let value = self
            .core
            .types()
            .get(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR type")])?;
        match value {
            CoreType::Unit => self.value_type(
                self.features
                    .mapping_for::<portable_build::UnitValues>()
                    .lower(&mut (), JavaUnitValuesInput::Type)?,
                "UnitValues type",
            ),
            CoreType::Bool => self.value_type(
                self.features
                    .mapping_for::<portable_build::BoolValues>()
                    .lower(&mut (), JavaBoolValuesInput::Type)?,
                "BoolValues type",
            ),
            CoreType::I32 => self.value_type(
                self.features
                    .mapping_for::<portable_build::I32Values>()
                    .lower(&mut (), JavaI32ValuesInput::Type)?,
                "I32Values type",
            ),
            CoreType::I64 => self.value_type(
                self.features
                    .mapping_for::<portable_build::I64Values>()
                    .lower(&mut (), JavaI64ValuesInput::Type)?,
                "I64Values type",
            ),
            CoreType::F64 => self.value_type(
                self.features
                    .mapping_for::<portable_build::F64Values>()
                    .lower(&mut (), JavaF64ValuesInput::Type)?,
                "F64Values type",
            ),
            CoreType::Char => self.value_type(
                self.features
                    .mapping_for::<portable_build::CharValues>()
                    .lower(&mut (), JavaCharValuesInput::Type)?,
                "CharValues type",
            ),
            CoreType::String => self.value_type(
                self.features
                    .mapping_for::<portable_build::TextValues>()
                    .lower(&mut (), JavaTextValuesInput::Type)?,
                "TextValues type",
            ),
            CoreType::Bytes => self.value_type(
                self.features
                    .mapping_for::<portable_build::BytesValues>()
                    .lower(&mut (), JavaBytesInput::Type)?,
                "BytesValues type",
            ),
            CoreType::List(inner) => self.value_type(
                self.features
                    .mapping_for::<portable_build::ListValues>()
                    .lower(
                        &mut (),
                        JavaListInput::Type {
                            element: self.ty(*inner)?,
                        },
                    )?,
                "ListValues type",
            ),
            CoreType::Option(inner) => self.value_type(
                self.features
                    .mapping_for::<portable_build::OptionValues>()
                    .lower(
                        &mut (),
                        JavaOptionInput::Type {
                            inner: self.ty(*inner)?,
                        },
                    )?,
                "OptionValues type",
            ),
            CoreType::Result { ok, error } => self.value_type(
                self.features
                    .mapping_for::<portable_build::ResultValues>()
                    .lower(
                        &mut (),
                        JavaResultInput::Type {
                            ok: self.ty(*ok)?,
                            error: self.ty(*error)?,
                        },
                    )?,
                "ResultValues type",
            ),
            CoreType::Record(id) => match self
                .features
                .mapping_for::<portable_build::Records>()
                .lower(
                    &mut (),
                    JavaRecordsInput::Type {
                        record: self.records[id],
                    },
                )? {
                JavaRecordsNode::Type(ty) => Ok(ty),
                JavaRecordsNode::Declaration(_) | JavaRecordsNode::Expression(_) => {
                    Err(vec![diagnostic(
                        "Java Records mapping returned a non-type for a record type",
                    )])
                }
            },
            CoreType::Enum(id) => match self.features.mapping_for::<portable_build::Enums>().lower(
                &mut (),
                JavaEnumsInput::Type {
                    enumeration: self.enums[id],
                },
            )? {
                JavaEnumsNode::Type(ty) => Ok(ty),
                JavaEnumsNode::Declaration(_)
                | JavaEnumsNode::Expression(_)
                | JavaEnumsNode::Statement(_) => Err(vec![diagnostic(
                    "Java Enums mapping returned a non-type for an enum type",
                )]),
            },
            CoreType::Interface(id) => match self
                .features
                .mapping_for::<portable_build::Interfaces>()
                .lower(
                    &mut (),
                    JavaInterfacesInput::Type {
                        interface: self.interfaces[id],
                    },
                )? {
                JavaInterfacesNode::Type(ty) => Ok(ty),
                JavaInterfacesNode::Declaration(_)
                | JavaInterfacesNode::Conformance(_)
                | JavaInterfacesNode::Expression(_) => Err(vec![diagnostic(
                    "Java Interfaces mapping returned a non-type for an interface type",
                )]),
            },
        }
    }

    pub(super) fn poly_result_type(&self, result: CoreTypeId) -> Result<JavaType, Vec<Diagnostic>> {
        Ok(JavaType::generic(
            JavaKnownType::RuntimeResult,
            vec![self.ty(result)?.boxed()],
        ))
    }
}
