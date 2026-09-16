//! Persistent registration state; function analysis exists only during body lowering.
use super::{Reader, Result, c, capabilities, origin};
use portable_backend_c::ast::*;
use portable_codegen::RustSourceNode;
use rustc_ast::BindingMode;
use rustc_hir::{
    self as hir,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

#[cfg(public_package_contract)]
#[path = "../../test/package_state_c.rs"]
pub(super) mod assertions;

pub(crate) struct State {
    pub registry: CRegistry,
    pub file: CFileRef,
    pub mappings: capabilities::CBindings,
    pub functions: HashMap<LocalDefId, CFunctionRef>,
    pub foreign_functions: HashMap<DefId, CFunctionRef>,
    pub header: Option<CFileRef>,
    pub constants: super::constants::OwnedConstants,
    pub records: HashMap<DefId, CStructRef>,
    pub declarations: Vec<CFileItem>,
    pub origins: origin::Cache,
}

impl State {
    fn into_reader<'tcx>(self, tcx: TyCtxt<'tcx>, root: LocalDefId) -> Result<Reader<'tcx>> {
        let function = self
            .functions
            .get(&root)
            .cloned()
            .ok_or("function body is absent from the C package registration")?;
        Ok(Reader {
            tcx,
            checked: tcx.typeck(root),
            mappings: self.mappings,
            registry: self.registry,
            file: self.file,
            function,
            functions: self.functions,
            foreign_functions: self.foreign_functions,
            header: self.header,
            constants: self.constants,
            records: self.records,
            declarations: self.declarations,
            origins: self.origins,
            root,
            bindings: HashMap::new(),
            control_scopes: HashMap::new(),
            parameters: vec![],
            next_binding: 0,
            next_scope: 0,
            active_scope: None,
            prelude: vec![],
            next_temporary: 0,
        })
    }

    fn from_reader(reader: Reader<'_>) -> Self {
        Self {
            registry: reader.registry,
            file: reader.file,
            mappings: reader.mappings,
            functions: reader.functions,
            foreign_functions: reader.foreign_functions,
            header: reader.header,
            constants: reader.constants,
            records: reader.records,
            declarations: reader.declarations,
            origins: reader.origins,
        }
    }
}

pub(super) struct Bodies {
    pub state: State,
    pub prototypes: Vec<CFileItem>,
    pub public_prototypes: Vec<CFileItem>,
    pub definitions: Vec<CFileItem>,
}

pub(super) fn lower_functions(
    tcx: TyCtxt<'_>,
    mut state: State,
    roots: &[LocalDefId],
    header: Option<&CFileRef>,
) -> Result<Bodies> {
    let mut prototypes = Vec::new();
    let mut public_prototypes = Vec::new();
    let mut definitions = Vec::new();
    for &id in roots {
        let mut reader = state.into_reader(tcx, id)?;
        reader.bind_parameters()?;
        let body = reader.branch(tcx.hir_body_owned_by(id).value, None)?;
        if !reader.prelude.is_empty() {
            return Err("undrained function evaluation prelude".into());
        }
        let linkage = if tcx.effective_visibilities(()).is_exported(id) {
            CLinkage::External
        } else {
            CLinkage::Internal
        };
        let primary = c(CDeclarations::new(
            &reader.registry,
            reader.function.file().clone(),
        ))?;
        let prototype = CFileItem::Declaration(c(
            primary.function_prototype(reader.function.clone(), linkage)
        )?);
        if header == Some(reader.function.file()) {
            public_prototypes.push(prototype);
        } else {
            prototypes.push(prototype);
        }
        let declarations = c(CDeclarations::new(&reader.registry, reader.file.clone()))?;
        definitions.push(CFileItem::Definition(c(declarations.function_definition(
            reader.function.clone(),
            linkage,
            reader.parameters.clone(),
            body,
        ))?));
        state = State::from_reader(reader);
    }
    Ok(Bodies {
        state,
        prototypes,
        public_prototypes,
        definitions,
    })
}

impl Reader<'_> {
    fn bind_parameters(&mut self) -> Result<()> {
        for (index, parameter) in self
            .tcx
            .hir_body_owned_by(self.root)
            .params
            .iter()
            .enumerate()
        {
            let hir::PatKind::Binding(BindingMode::NONE, id, _, None) = parameter.pat.kind else {
                return Err("only plain immutable parameters are implemented".into());
            };
            let name = format!("v{index}");
            let key = self.key(id, RustSourceNode::Parameter(id.local_id.as_u32()), &name)?;
            let parameter = c(self.registry.register_parameter(
                &self.function,
                index,
                key,
                CConstness::Unqualified,
            ))?;
            self.bindings
                .insert(id, c(self.expressions().parameter(parameter.clone()))?);
            self.parameters.push(parameter);
            self.next_binding += 1;
        }
        Ok(())
    }
}
