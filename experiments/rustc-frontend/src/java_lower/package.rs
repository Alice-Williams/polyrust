//! Package authority and resource budgets outlive individual function Readers.
use super::{Callable, Place, Reader, Result, Value, capabilities, records};
use portable_backend_java::{
    ast::*,
    dialect::{JavaDialect, JavaImportedCallable},
};
use portable_codegen::TargetAstBuilder;
use rustc_ast::BindingMode;
use rustc_hir::{
    self as hir,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

#[cfg(java_ast_probe)]
#[path = "../../test/package_state_java.rs"]
pub(super) mod assertions;

pub(crate) struct State {
    pub builder: TargetAstBuilder<JavaDialect>,
    pub mappings: capabilities::JavaBindings,
    pub functions: HashMap<LocalDefId, Callable>,
    pub imported: HashMap<DefId, JavaImportedCallable>,
    pub public_api: bool,
    pub constants: HashMap<DefId, super::constants::Constant>,
    pub foreign_constants: HashMap<DefId, portable_backend_java::dialect::JavaImportedValue>,
    pub records: HashMap<DefId, records::Record>,
    pub origins: crate::source_origin::Cache,
    pub remaining: usize,
}

impl State {
    fn into_reader<'tcx>(self, tcx: TyCtxt<'tcx>, root: LocalDefId) -> Result<Reader<'tcx>> {
        if !self.functions.contains_key(&root) {
            return Err("function body is absent from the Java package registration".into());
        }
        Ok(Reader {
            tcx,
            checked: tcx.typeck(root),
            mappings: self.mappings,
            builder: self.builder,
            functions: self.functions,
            imported: self.imported,
            public_api: self.public_api,
            constants: self.constants,
            foreign_constants: self.foreign_constants,
            records: self.records,
            origins: self.origins,
            remaining: self.remaining,
            bindings: HashMap::new(),
            prelude: vec![],
            next_local: 0,
            active_scope: None,
            scopes: HashMap::new(),
            depth: 0,
            #[cfg(java_ast_probe)]
            expression_observations: Vec::new(),
        })
    }

    fn from_reader(reader: Reader<'_>) -> Self {
        Self {
            builder: reader.builder,
            mappings: reader.mappings,
            functions: reader.functions,
            imported: reader.imported,
            public_api: reader.public_api,
            constants: reader.constants,
            foreign_constants: reader.foreign_constants,
            records: reader.records,
            origins: reader.origins,
            remaining: reader.remaining,
        }
    }
}

pub(super) fn lower_functions(
    tcx: TyCtxt<'_>,
    mut state: State,
    roots: &[LocalDefId],
) -> Result<(State, Vec<JavaMember>)> {
    let mut members = Vec::new();
    for &id in roots {
        let mut reader = state.into_reader(tcx, id)?;
        let callable = reader.functions[&id].clone();
        let parameters = reader.bind_parameters(id)?;
        let body = reader.branch(tcx.hir_body_owned_by(id).value, None)?;
        #[cfg(java_ast_probe)]
        super::assertions::function(&reader, id, &body);
        members.push(JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Callable(callable.id),
            annotations: vec![],
            modifiers: vec![
                match callable.visibility {
                    JavaVisibility::Public => JavaModifier::Public,
                    _ => JavaModifier::Private,
                },
                JavaModifier::Static,
            ],
            type_parameters: vec![],
            return_type: callable.signature.result,
            name: callable.name,
            parameters,
            body: Some(body),
        }));
        state = State::from_reader(reader);
    }
    Ok((state, members))
}

impl Reader<'_> {
    fn bind_parameters(&mut self, root: LocalDefId) -> Result<Vec<JavaParameter>> {
        let mut parameters = Vec::new();
        let signature = self.functions[&root].signature.clone();
        let body = self.tcx.hir_body_owned_by(root);
        if body.params.len() != signature.parameters.len() {
            return Err("source parameter count mismatch".into());
        }
        for (parameter, ty) in body.params.iter().zip(signature.parameters) {
            let hir::PatKind::Binding(BindingMode::NONE, id, _, None) = parameter.pat.kind else {
                return Err("only plain immutable parameters are implemented".into());
            };
            let spelling = self.fresh()?;
            let plan = self.ty(self.checked.node_type(parameter.pat.hir_id))?;
            if plan.java_type() != ty {
                return Err("parameter source type differs from registered Java signature".into());
            }
            self.bindings.insert(
                id,
                Place::resolved(Value::new(
                    plan,
                    JavaExpr::local(ty.clone(), spelling.clone()),
                )?),
            );
            parameters.push(JavaParameter {
                ty,
                name: spelling,
                final_parameter: true,
            });
        }
        Ok(parameters)
    }
}
