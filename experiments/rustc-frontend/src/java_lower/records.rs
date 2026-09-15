//! Source-owned immutable nominal declarations and exact field references.
use super::{Reader, Result, TypePlan, name, source};
use crate::source_origin;
use portable_backend_java::ast::*;
use portable_codegen::{GeneratedOrigin, GeneratedType, GeneratedTypeId, RustSourceNode};
use rustc_middle::ty::{self, AdtDef, GenericArgsRef};

#[derive(Clone)]
#[cfg_attr(local_constant_ast_probe, derive(Debug))]
pub(super) struct Record {
    pub id: GeneratedTypeId,
    pub declaration: JavaTypeDeclaration,
    pub fields: Vec<JavaFieldRef>,
}

impl<'tcx> Reader<'tcx> {
    pub(super) fn record(
        &mut self,
        definition: AdtDef<'tcx>,
        arguments: GenericArgsRef<'tcx>,
    ) -> Result<TypePlan> {
        if let Some(record) = self.records.get(&definition.did()) {
            return Ok(TypePlan::Record(record.id));
        }
        if !definition.did().is_local() || definition.non_enum_variant().fields.is_empty() {
            return Err(
                "only local nonempty immutable scalar-field records are implemented".into(),
            );
        }
        let identity = source_origin::identity(self.tcx, definition.did());
        let spelling = name(&format!("Record{:016x}", identity.definition_path_hash))?;
        let origin = source_origin::read(
            self.tcx,
            &mut self.origins,
            definition.did(),
            RustSourceNode::Declaration,
            self.tcx.def_span(definition.did()),
        )?;
        #[cfg(java_ast_probe)]
        super::assertions::origin(self.tcx, definition.did(), &origin);
        let id = self.builder.generated_type(GeneratedType {
            name: spelling.as_str().into(),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Private,
            origin: GeneratedOrigin::RustSource(origin),
            source: source(),
        });
        let mut components = Vec::new();
        let mut parameters = Vec::new();
        let mut assignments = Vec::new();
        let mut fields = Vec::new();
        for (index, field) in definition.non_enum_variant().fields.iter().enumerate() {
            let field_type = self
                .tcx
                .try_normalize_erasing_regions(
                    ty::TypingEnv::fully_monomorphized(),
                    field.ty(self.tcx, arguments),
                )
                .map_err(|_| "record field normalization failed")?;
            let plan = match field_type.kind() {
                ty::Int(ty::IntTy::I32) => TypePlan::I32,
                ty::Int(ty::IntTy::I64) => TypePlan::I64,
                ty::Bool => TypePlan::Bool,
                _ => return Err("only scalar record fields are implemented".into()),
            };
            let ty = plan.java_type();
            let field_name = name(&format!("f{index}"))?;
            let origin = source_origin::read(
                self.tcx,
                &mut self.origins,
                field.did,
                RustSourceNode::Declaration,
                self.tcx.def_span(field.did),
            )?;
            let reference = JavaFieldRef::RustSource {
                owner: id,
                field: origin.declaration,
                name: field_name.clone(),
                ty: ty.clone(),
            };
            components.push(JavaRecordComponent {
                origin: JavaRecordComponentOrigin::RustSource(JavaSourceFieldOrigin {
                    owner: identity,
                    origin,
                }),
                ty: ty.clone(),
                name: field_name.clone(),
            });
            parameters.push(JavaParameter {
                ty: ty.clone(),
                name: field_name.clone(),
                final_parameter: true,
            });
            assignments.push(JavaStmt::Assign {
                target: JavaExpr {
                    ty: ty.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Field {
                        receiver: Box::new(JavaExpr {
                            ty: TypePlan::Record(id).java_type(),
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::Value(JavaValueRef::This),
                        }),
                        field: reference.clone(),
                    },
                },
                value: JavaExpr::local(ty, field_name),
            });
            fields.push(reference);
        }
        let declaration = JavaTypeDeclaration {
            declared: Some(id),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Private,
            modifiers: vec![],
            name: spelling.clone(),
            type_parameters: vec![],
            record_components: components,
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Private],
                name: spelling,
                parameters,
                body: JavaBlock::new(assignments),
            })],
        };
        #[cfg(java_ast_probe)]
        super::assertions::record(self.tcx, definition, &declaration);
        self.records.insert(
            definition.did(),
            Record {
                id,
                declaration,
                fields,
            },
        );
        Ok(TypePlan::Record(id))
    }
}
