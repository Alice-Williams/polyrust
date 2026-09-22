//! Original scalar identities reconciled with the exact certified C inventory.
use super::{
    ApiManifest,
    serialization::{identity, quote},
};
use portable_backend_c::{ast::*, dialect::CDialect};
use portable_codegen::{
    RenderReadyPackage, RustDeclarationId, RustFunctionTypes, RustResultKind, RustScalarKind,
    RustSourceNode, RustSourceTypes,
};
use std::collections::BTreeMap;
use std::fmt::Write;

fn scalar(kind: RustScalarKind) -> CObjectType {
    CObjectType::scalar(match kind {
        RustScalarKind::I32 => CScalarType::I32,
        RustScalarKind::I64 => CScalarType::I64,
        RustScalarKind::Bool => CScalarType::Bool,
        RustScalarKind::F64 => CScalarType::F64,
        RustScalarKind::Char => CScalarType::U32,
    })
}

impl ApiManifest {
    #[allow(
        dead_code,
        reason = "Standalone C manifest probes do not resolve dependency calls"
    )]
    pub(crate) fn source_signature(&self, id: RustDeclarationId) -> Option<&RustFunctionTypes> {
        self.source_types.as_ref()?.functions().get(&id)
    }

    pub(super) fn copy_source_types(
        mut self,
        package: &RenderReadyPackage<CDialect>,
        types: Option<&RustSourceTypes>,
    ) -> Result<Self, String> {
        if let Some(types) = types {
            self = self.with_source_types(package, types.clone())?;
        }
        Ok(self)
    }

    pub(crate) fn with_source_types(
        mut self,
        package: &RenderReadyPackage<CDialect>,
        types: RustSourceTypes,
    ) -> Result<Self, String> {
        if types.root() != self.exports.root || types.functions().len() != self.functions.len() {
            return Err("C source type owner or function inventory differs".into());
        }
        for (id, function) in &self.functions {
            let original = types
                .functions()
                .get(id)
                .ok_or("C source function type facts missing")?;
            let signature = function.reference.signature();
            let result_matches = match (original.result, signature.return_type()) {
                (RustResultKind::Unit, CReturnType::Void) => true,
                (RustResultKind::Scalar(kind), CReturnType::Value(value)) => {
                    scalar(kind) == *value.declared_type()
                }
                _ => false,
            };
            if !result_matches
                || original.parameters.len() != signature.parameters().len()
                || !original
                    .parameters
                    .iter()
                    .zip(signature.parameters())
                    .all(|(kind, parameter)| scalar(*kind) == *parameter.declared_type())
            {
                return Err("C source function types disagree with target signature".into());
            }
        }
        let mut fields = BTreeMap::new();
        for member in portable_backend_c::dialect::c_defined_members(package) {
            let CGeneratedOrigin::RustSource(origin) = &member.key().origin else {
                return Err("C source field lacks original declaration".into());
            };
            let original = types
                .fields()
                .get(&origin.declaration)
                .ok_or("C source field type facts missing")?;
            let CGeneratedOrigin::RustSource(owner) = &member.owner().key().origin else {
                return Err("C source field owner lacks original declaration".into());
            };
            if origin.node != RustSourceNode::Declaration
                || origin.crate_exports != self.exports
                || owner.node != RustSourceNode::Declaration
                || owner.crate_exports != self.exports
                || owner.declaration != original.owner
                || scalar(original.kind) != *member.ty()
                || fields.insert(origin.declaration, *original).is_some()
            {
                return Err("C source field facts disagree with target declaration".into());
            }
        }
        if &fields != types.fields() {
            return Err("C source field inventory differs".into());
        }
        self.source_types = Some(types);
        self.encoded_bound()?;
        Ok(self)
    }

    pub(super) fn has_characters(&self) -> bool {
        self.source_types
            .as_ref()
            .is_some_and(RustSourceTypes::contains_char)
    }

    pub(super) fn source_type_bound(&self) -> Result<usize, String> {
        let Some(types) = self
            .source_types
            .as_ref()
            .filter(|types| types.contains_char())
        else {
            return Ok(0);
        };
        let mut bound = 512usize;
        for function in types.functions().values() {
            bound = function
                .parameters
                .len()
                .checked_mul(8)
                .and_then(|parameters| parameters.checked_add(160))
                .and_then(|bytes| bound.checked_add(bytes))
                .ok_or("C source type metadata overflow")?;
        }
        types
            .fields()
            .len()
            .checked_mul(160)
            .and_then(|bytes| bound.checked_add(bytes))
            .ok_or_else(|| "C source field metadata overflow".into())
    }

    pub(super) fn write_source_types(&self, text: &mut String) {
        let Some(types) = self
            .source_types
            .as_ref()
            .filter(|types| types.contains_char())
        else {
            return;
        };
        text.push_str(",\"source_types\":{\"char_foreign_input_domain\":\"Unicode scalar: 0..=0x10ffff excluding 0xd800..=0xdfff\",\"functions\":[");
        for (index, (id, signature)) in types.functions().iter().enumerate() {
            if index != 0 {
                text.push(',');
            }
            write!(
                text,
                "{{\"id\":{},\"result\":{},\"parameters\":[",
                identity(*id),
                quote(signature.result.spelling())
            )
            .unwrap();
            for (index, kind) in signature.parameters.iter().enumerate() {
                if index != 0 {
                    text.push(',');
                }
                text.push_str(&quote(kind.spelling()));
            }
            text.push_str("]}");
        }
        text.push_str("],\"fields\":[");
        for (index, (id, field)) in types.fields().iter().enumerate() {
            if index != 0 {
                text.push(',');
            }
            write!(
                text,
                "{{\"id\":{},\"owner\":{},\"scalar\":{}}}",
                identity(*id),
                identity(field.owner),
                quote(field.kind.spelling())
            )
            .unwrap();
        }
        text.push_str("]}");
    }
}
