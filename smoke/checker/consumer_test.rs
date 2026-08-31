#![forbid(unsafe_code)]

use portable_check::v0::check_program;
use portable_ir::v0::{Document, IrVersion, Module};

#[test]
fn downstream_code_obtains_checked_program_only_through_checker() {
    let checked = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "consumer".to_owned(),
            declarations: vec![],
        },
    ))
    .expect("empty module checks");

    assert_eq!(checked.module().name, "consumer");
    assert!(checked.capabilities().program().is_empty());
}
