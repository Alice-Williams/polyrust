//! Exact source export/declaration reconciliation for a certified Java owner.
mod constants;
use super::{JavaDialect, bodies, exports::Agreement, records};
use crate::ast::{
    JavaDeclarationKind, JavaDeclaredPath, JavaFileItem, JavaFilePlacement, JavaHeritage,
    JavaMember, JavaMethod, JavaMethodDeclaration, JavaMethodSignature, JavaModifier, JavaPackage,
    JavaPrimitive, JavaResolvedName, JavaSourceDeclaration, JavaType, JavaVisibility,
};
use portable_codegen::{
    GeneratedOrigin, GeneratedSymbolId, RenderReadyPackage, RustCrateExports, RustDeclarationId,
    RustExportNamespace, RustExportTarget, RustSourceOrigin, RustVisibility, SourceRole,
    SynthesisReason, TargetSymbolRef,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(super) struct Function {
    pub generated: portable_codegen::GeneratedCallableId,
    pub call_height: usize,
    pub source: Arc<RustSourceOrigin>,
    pub path: JavaDeclaredPath,
    pub signature: JavaMethodSignature,
}
pub(super) struct Constant {
    pub generated: portable_codegen::GeneratedValueId,
    pub source: Arc<RustSourceOrigin>,
    pub path: JavaDeclaredPath,
    pub ty: JavaType,
    pub value: crate::ast::JavaScalarConstantValue,
}
pub(super) struct Inventory {
    pub root: RustDeclarationId,
    pub namespace: JavaPackage,
    pub functions: BTreeMap<RustDeclarationId, Function>,
    pub constants: BTreeMap<RustDeclarationId, Constant>,
    pub foreign_constants: Vec<super::JavaForeignConstantExport>,
    pub result_families:
        BTreeMap<portable_codegen::GeneratedTypeId, Arc<super::result_layout::ResultLayout>>,
}

pub(super) fn scalar(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Primitive(
            JavaPrimitive::Int
                | JavaPrimitive::Long
                | JavaPrimitive::Boolean
                | JavaPrimitive::Double
        )
    )
}

pub(super) fn signature(method: &JavaMethod) -> JavaMethodSignature {
    JavaMethodSignature {
        receiver: None,
        parameters: method
            .parameters
            .iter()
            .map(|value| value.ty.clone())
            .collect(),
        result: method.return_type.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    }
}

pub(super) fn collect_with_results(
    package: &RenderReadyPackage<JavaDialect>,
    selections: &[super::super::JavaScalarResultTypes],
) -> Result<Inventory, String> {
    collect_results_with_budget(package, selections, &mut bodies::Budget::new())
}

#[cfg(test)]
pub(super) fn collect_with_budget(
    package: &RenderReadyPackage<JavaDialect>,
    budget: &mut bodies::Budget,
) -> Result<Inventory, String> {
    collect_results_with_budget(package, &[], budget)
}

fn collect_results_with_budget(
    package: &RenderReadyPackage<JavaDialect>,
    selections: &[super::super::JavaScalarResultTypes],
    budget: &mut bodies::Budget,
) -> Result<Inventory, String> {
    let result_families = super::result_layout::collect(package, selections)?;
    let result_profile = super::result_profile::ResultProfile::new(&result_families);
    let selected_types = result_families
        .values()
        .flat_map(|layout| {
            [
                layout.types.interface,
                layout.types.success,
                layout.types.error,
            ]
        })
        .collect::<BTreeSet<_>>();
    let [file] = package.ast().files() else {
        return Err("Java dependency API requires exactly one owning compilation unit".into());
    };
    let JavaPackage::RustCrate(crate_id) = *file.module() else {
        return Err("Java dependency API requires a RustCrate namespace".into());
    };
    if file.role() != SourceRole::PublicApi
        || *file.placement() != JavaFilePlacement::Main
        || file.path().as_str()
            != format!(
                "{}Generated.java",
                file.module().source_directory(JavaFilePlacement::Main)
            )
    {
        return Err("Java dependency API requires the canonical public Generated.java file".into());
    }
    let [item] = file.items() else {
        return Err("Java dependency API requires one complete facade item".into());
    };
    let JavaFileItem::Type {
        declaration: facade,
        ..
    } = &item.item
    else {
        return Err("Java dependency API requires a source facade".into());
    };
    let facade_id = facade
        .declared
        .ok_or("Java dependency facade has no registered identity")?;
    let Some(JavaSourceDeclaration::Type(registration)) = item
        .source_inventory
        .get(GeneratedSymbolId::Type(facade_id))
    else {
        return Err("Java dependency facade has no retained registration".into());
    };
    if !matches!(
        registration.origin,
        GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint)
    ) || registration.name != "Generated"
        || registration.kind != JavaDeclarationKind::FinalClass
        || registration.visibility != JavaVisibility::Public
        || facade.name.as_str() != "Generated"
        || facade.kind != JavaDeclarationKind::FinalClass
        || facade.visibility != JavaVisibility::Public
        || !facade.modifiers.is_empty()
        || !facade.type_parameters.is_empty()
        || !facade.record_components.is_empty()
        || facade.heritage != JavaHeritage::None
        || !facade.permits.is_empty()
    {
        return Err("Java dependency facade identity or shape disagrees".into());
    }
    let JavaFileItem::Type {
        source_package,
        dependencies,
        ..
    } = &item.item
    else {
        unreachable!()
    };
    let exports = match source_package {
        Some(source) => source.exports(),
        None => {
            &item
                .source_inventory
                .iter()
                .find_map(|(_, value)| match value.origin() {
                    GeneratedOrigin::RustSource(origin) => Some(origin),
                    _ => None,
                })
                .ok_or("Java dependency API has no source declarations")?
                .crate_exports
        }
    };
    let selected = if source_package.is_some() {
        super::super::constant_exports::collect(exports, dependencies)?
    } else {
        super::super::constant_exports::Selection::default()
    };
    if exports.root.crate_id != crate_id {
        return Err("Java dependency export owner disagrees".into());
    }
    let public = public_bindings(
        exports,
        &selected,
        source_package.is_some() && !result_families.is_empty(),
    )?;
    let mut agreement = Agreement::new(exports);
    let mut expected = BTreeSet::from([GeneratedSymbolId::Type(facade_id)]);
    for (_, value) in item.source_inventory.iter() {
        if let GeneratedOrigin::RustSource(origin) = value.origin() {
            if origin.declaration.crate_id != crate_id {
                return Err("Java dependency registrations disagree on source owner".into());
            }
            agreement.check(&origin.crate_exports)?;
        }
    }
    let mut functions = BTreeMap::new();
    let mut constants = BTreeMap::new();
    let mut constant_types = BTreeMap::new();
    let mut methods = BTreeMap::new();
    let mut nominal = BTreeMap::new();
    let mut constructors = 0;
    for member in &facade.members {
        match member {
            JavaMember::Constructor(constructor)
                if constructor.modifiers == [JavaModifier::Private]
                    && constructor.name == facade.name
                    && constructor.parameters.is_empty()
                    && constructor.body.statements.is_empty() =>
            {
                budget.charge(0)?; // The empty facade constructor's body block.
                constructors += 1;
            }
            JavaMember::NestedType(record) => {
                if record
                    .declared
                    .is_some_and(|id| selected_types.contains(&id))
                {
                    budget.charge(0)?;
                    let id = record.declared.expect("selected declaration");
                    if !expected.insert(GeneratedSymbolId::Type(id)) {
                        return Err(
                            "Java dependency duplicates a selected result declaration".into()
                        );
                    }
                    continue;
                }
                let id = records::verify(record, &item.source_inventory, &mut agreement, budget)?;
                if !expected.insert(GeneratedSymbolId::Type(id))
                    || nominal.insert(id, record).is_some()
                {
                    return Err("Java dependency duplicates a nominal declaration".into());
                }
            }
            JavaMember::Field(field) => {
                budget.charge(0)?;
                let constant =
                    constants::verify(field, item, *file.module(), &facade.name, &public)?;
                if !expected.insert(GeneratedSymbolId::Value(constant.generated))
                    || constant_types
                        .insert(constant.generated, constant.ty.clone())
                        .is_some()
                    || constants
                        .insert(constant.source.declaration, constant)
                        .is_some()
                {
                    return Err("Java dependency duplicates a source constant".into());
                }
            }
            JavaMember::Method(method) => {
                let JavaMethodDeclaration::Callable(id) = method.declared else {
                    return Err("Java dependency method is not a source callable".into());
                };
                let symbol = GeneratedSymbolId::Callable(id);
                let Some(JavaSourceDeclaration::Callable(registration)) =
                    item.source_inventory.get(symbol)
                else {
                    return Err("Java dependency callable has no source registration".into());
                };
                let GeneratedOrigin::RustSource(origin) = &registration.origin else {
                    return Err("Java dependency callable lacks source provenance".into());
                };
                let exported = public.contains(&origin.declaration);
                let visibility = if exported {
                    JavaVisibility::Public
                } else {
                    JavaVisibility::Private
                };
                let modifier = if exported {
                    JavaModifier::Public
                } else {
                    JavaModifier::Private
                };
                let signature = signature(method);
                if registration.visibility != visibility
                    || origin.externally_reachable != exported
                    || (exported && origin.visibility != RustVisibility::Public)
                    || method.modifiers.len() != 2
                    || !method.modifiers.contains(&modifier)
                    || !method.modifiers.contains(&JavaModifier::Static)
                    || !method.annotations.is_empty()
                    || !method.type_parameters.is_empty()
                    || !(scalar(&method.return_type)
                        || result_profile.role(&method.return_type).is_some()
                        || method.return_type == JavaType::primitive(JavaPrimitive::Void))
                    || method.parameters.iter().any(|value| {
                        !(scalar(&value.ty) || result_profile.role(&value.ty).is_some())
                            || !value.final_parameter
                    })
                    || registration.signature != JavaDialect.coarse_signature(&signature)
                    || registration.name != method.name.as_str()
                    || method.body.is_none()
                    || !expected.insert(symbol)
                    || methods.insert(id, method).is_some()
                {
                    return Err("Java dependency callable visibility/signature/declaration inventory disagrees".into());
                }
                let Some(JavaResolvedName::DeclaredPath(path)) =
                    item.names.get(&TargetSymbolRef::Generated(symbol))
                else {
                    return Err(
                        "Java dependency callable lacks its resolved declaration path".into(),
                    );
                };
                if path.package != *file.module()
                    || path.owners != [facade.name.clone()]
                    || path.member != method.name
                {
                    return Err("Java dependency callable resolved owner disagrees".into());
                }
                if exported
                    && functions
                        .insert(
                            origin.declaration,
                            Function {
                                generated: id,
                                call_height: 0,
                                source: origin.clone(),
                                path: path.clone(),
                                signature,
                            },
                        )
                        .is_some()
                {
                    return Err("Java dependency duplicates a public source declaration".into());
                }
            }
            _ => return Err("Java dependency facade contains an unadmitted member".into()),
        }
    }
    if constructors != 1 {
        return Err("Java dependency facade requires one private empty constructor".into());
    }
    if item
        .source_inventory
        .iter()
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>()
        != expected
    {
        return Err("Java dependency contains unsupported or missing source declarations".into());
    }
    if functions.keys().any(|id| constants.contains_key(id))
        || functions
            .keys()
            .chain(constants.keys())
            .copied()
            .collect::<BTreeSet<_>>()
            != public
    {
        return Err("Java dependency API omits a compiler public binding".into());
    }
    let heights =
        bodies::verify_with_results(&methods, &nominal, &constant_types, &result_profile, budget)?;
    for function in functions.values_mut() {
        function.call_height = heights[&function.generated];
    }
    Ok(Inventory {
        result_families,
        root: exports.root,
        namespace: *file.module(),
        functions,
        constants,
        foreign_constants: selected
            .foreign
            .into_iter()
            .map(|((module, name), value)| {
                super::JavaForeignConstantExport::new(module, name, value.constant().clone())
            })
            .collect(),
    })
}

fn public_bindings(
    exports: &RustCrateExports,
    selected: &super::super::constant_exports::Selection,
    has_selected_families: bool,
) -> Result<BTreeSet<RustDeclarationId>, String> {
    let mut public = BTreeSet::new();
    for (module, bindings) in &exports.modules {
        for (name, target) in bindings {
            match target {
                RustExportTarget::Module(id)
                    if name.namespace == RustExportNamespace::Type
                        && id.crate_id == exports.root.crate_id
                        && exports.modules.contains_key(id) => {}
                RustExportTarget::Declaration(id)
                    if name.namespace == RustExportNamespace::Value
                        && id.crate_id == exports.root.crate_id =>
                {
                    public.insert(*id);
                }
                RustExportTarget::Declaration(id)
                    if name.namespace == RustExportNamespace::Value
                        && selected
                            .foreign
                            .get(&(*module, name.clone()))
                            .is_some_and(|value| value.constant().declaration() == *id) => {}
                _ => {
                    return Err(
                        "Java dependency export has no supported local scalar function/constant mapping"
                            .into(),
                    );
                }
            }
        }
    }
    if public.is_empty() && selected.foreign.is_empty() && !has_selected_families {
        return Err("Java dependency API has no public function/constant bindings".into());
    }
    Ok(public)
}
