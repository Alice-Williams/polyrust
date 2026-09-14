//! Register source declarations, lower bodies, and assemble the existing AST.
use super::{
    Callable, Place, Reader, Result, Selection, TypePlan, Value, capabilities, functions, name,
    source,
};
use crate::source_admission as admission;
use crate::source_origin;
use capabilities::{
    EntryInput, EntrySignatures, FunctionInput, FunctionSignatures, Mapping, Supports,
};
use portable_backend_java::{
    ast::*,
    dialect::{JavaDialect, JavaInvocationKind},
};
use portable_codegen::*;
use rustc_ast::BindingMode;
use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

pub(super) fn lower(
    tcx: TyCtxt<'_>,
    selection: Selection,
    lookup: Option<&super::DependencyLookup<'_>>,
) -> Result<TargetAstPackage<JavaDialect>> {
    let mappings = capabilities::java_bindings();
    let entry = match selection {
        Selection::Entry(root) => {
            let signature = Supports::<EntrySignatures>::mapping(&mappings)
                .lower(&mut (), EntryInput { tcx, root })?;
            if !tcx.effective_visibilities(()).is_exported(root) {
                return Err("selected entry must be externally reachable in Rust".into());
            }
            Some((root, signature))
        }
        Selection::PublicApi => None,
    };
    admission::aliases(tcx)?;
    let mut origins = source_origin::Cache::default();
    let exports = origins.exports(tcx)?;
    let roots = match &entry {
        Some((root, _)) => vec![*root],
        None => functions::public_roots(tcx, &exports)?,
    };
    let inventory = functions::inventory(tcx, &roots)?;
    let imported = super::foreign::register(tcx, &inventory.foreign, &mappings, lookup)?;
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let facade = builder.generated_type(GeneratedType {
        name: "Generated".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
        source: source(),
    });
    let mut functions = HashMap::new();
    for id in &inventory.local {
        let signature = if let Some((selected, signature)) = &entry
            && id == selected
        {
            signature.clone()
        } else {
            Supports::<FunctionSignatures>::mapping(&mappings).lower(
                &mut (),
                FunctionInput {
                    tcx,
                    function: id.to_def_id(),
                },
            )?
        };
        let origin = source_origin::read(
            tcx,
            &mut origins,
            id.to_def_id(),
            RustSourceNode::Declaration,
            tcx.def_span(*id),
        )?;
        let spelling = if entry.as_ref().is_some_and(|(root, _)| root == id) {
            name("score")?
        } else {
            name(&format!(
                "fn{:016x}",
                origin.declaration.definition_path_hash
            ))?
        };
        let visibility = if origin.externally_reachable {
            JavaVisibility::Public
        } else {
            JavaVisibility::Private
        };
        let symbol = builder.callable(GeneratedCallable {
            name: spelling.as_str().into(),
            signature: target_signature(&signature)?,
            visibility,
            origin: GeneratedOrigin::RustSource(origin),
            source: source(),
        });
        functions.insert(
            *id,
            Callable {
                id: symbol,
                name: spelling,
                signature,
                visibility,
            },
        );
    }
    let mut reader = Reader {
        tcx,
        checked: tcx.typeck(roots[0]),
        mappings,
        builder,
        functions,
        imported: imported.functions,
        records: HashMap::new(),
        bindings: HashMap::new(),
        origins,
        prelude: vec![],
        next_local: 0,
        active_scope: None,
        scopes: HashMap::new(),
        depth: 0,
        remaining: 100_000,
        #[cfg(java_ast_probe)]
        expression_observations: Vec::new(),
    };
    let mut members = vec![JavaMember::Constructor(JavaConstructor {
        modifiers: vec![JavaModifier::Private],
        name: name("Generated")?,
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    })];
    for id in inventory.local {
        let callable = reader.functions[&id].clone();
        let parameters = reader.begin_function(id)?;
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
    }
    let mut declared = vec![GeneratedSymbolId::Type(facade)];
    let mut callables: Vec<_> = reader
        .functions
        .values()
        .map(|function| function.id)
        .collect();
    callables.sort();
    declared.extend(callables.into_iter().map(GeneratedSymbolId::Callable));
    let mut records: Vec<_> = reader.records.into_values().collect();
    records.sort_by_key(|record| record.id);
    for record in records {
        declared.push(GeneratedSymbolId::Type(record.id));
        members.push(JavaMember::NestedType(record.declaration));
    }
    let declaration = JavaTypeDeclaration {
        declared: Some(facade),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: name("Generated")?,
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members,
    };
    let package = JavaPackage::RustCrate(exports.root.crate_id);
    let path = RelativeOutputPath::new(format!(
        "{}Generated.java",
        package.source_directory(JavaFilePlacement::Main)
    ))
    .map_err(|error| format!("Java package path: {error:?}"))?;
    let file = reader.builder.file(TargetFile::new(
        path,
        SourceRole::PublicApi,
        package,
        JavaFilePlacement::Main,
        vec![JavaFileItem::Type {
            declared,
            conformances: JavaConformanceInventory::structural().into(),
            dependencies: imported.bindings,
            declaration: Box::new(declaration),
        }],
        JavaSourceFileKind::CompilationUnit,
        source(),
    ));
    reader.builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        vec![TargetFileMember::Source(file)],
        source(),
    ));
    let package = reader.builder.build();
    #[cfg(java_ast_probe)]
    super::assertions::package(tcx, &package);
    Ok(package)
}

fn target_signature(
    signature: &JavaMethodSignature,
) -> Result<TargetCallableSignature<JavaDialect>> {
    let ty = |ty: &JavaType| match TypePlan::scalar(ty)? {
        TypePlan::I32 => Ok(TargetTypeRef::Primitive(JavaPrimitive::Int)),
        TypePlan::Bool => Ok(TargetTypeRef::Primitive(JavaPrimitive::Boolean)),
        _ => Err("non-scalar callable signature".to_owned()),
    };
    Ok(TargetCallableSignature {
        invocation: JavaInvocationKind::Static,
        receiver: None,
        parameters: signature.parameters.iter().map(ty).collect::<Result<_>>()?,
        return_type: ty(&signature.result)?,
    })
}

impl Reader<'_> {
    fn begin_function(&mut self, root: LocalDefId) -> Result<Vec<JavaParameter>> {
        self.checked = self.tcx.typeck(root);
        self.bindings.clear();
        self.scopes.clear();
        self.prelude.clear();
        #[cfg(java_ast_probe)]
        self.expression_observations.clear();
        self.active_scope = None;
        self.next_local = 0;
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
            let plan = TypePlan::scalar(&ty)?;
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
