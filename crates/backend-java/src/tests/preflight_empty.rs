//! Backend service prerequisites exist even without semantic feature uses.
use super::JavaCapabilityRegistry;
use crate::capabilities::java_capabilities;
use portable_build::{CapabilityId, Modules, PortableTests};
use portable_build::{ModuleBuilder, portable_name, typed_program};
use portable_codegen::collect_core_features;
use portable_codegen::{Backend, BackendOptions};
use portable_core_ir::lower_checked;

#[test]
fn genuinely_empty_program_certifies_every_unconditional_java_service() {
    let checked = ModuleBuilder::new("empty_admission").finish().unwrap();
    let core = lower_checked(&checked).unwrap();
    assert!(collect_core_features(&core).is_empty());
    let selection = JavaCapabilityRegistry::default().select(&core).unwrap();
    assert!(selection.selected.is_empty());
    assert_eq!(
        selection.program_prerequisites,
        vec![CapabilityId::Modules, CapabilityId::PortableTests]
    );
    for missing in 0..selection.program_prerequisites.len() {
        let mut corrupted = selection.clone();
        corrupted.program_prerequisites.remove(missing);
        assert!(corrupted.validate_for(&core, java_capabilities()).is_err());
    }
    let mut extra = selection.clone();
    extra.program_prerequisites.push(CapabilityId::Functions);
    assert!(extra.validate_for(&core, java_capabilities()).is_err());

    crate::capabilities::reset_java_mapping_invocations();
    crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let expected = std::collections::BTreeSet::from([
        std::any::type_name::<Modules>(),
        std::any::type_name::<PortableTests>(),
    ]);
    assert_eq!(crate::capabilities::java_mapping_invocations(), expected);

    // PortableTests is a Java-generated harness service, not an invented
    // generic semantic requirement for a program containing no user tests.
    let typed = typed_program(portable_name!("empty_typed"), |builder| builder);
    crate::capabilities::reset_java_mapping_invocations();
    crate::JavaBackend
        .generate_typed(&typed)
        .expect("Java resource capacity");
    assert_eq!(crate::capabilities::java_mapping_invocations(), expected);
}
