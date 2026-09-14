//! Fail-closed process stack control shared by every native frame probe.
use std::{
    path::Path,
    process::{Command, ExitStatus},
};

const LAUNCH: &str =
    "ulimit -s 1024 || exit 125; test \"$(ulimit -s)\" = 1024 || exit 126; exec \"$1\"";

pub(super) fn run(binary: &Path) -> ExitStatus {
    Command::new("bash")
        .args(["-c", LAUNCH, "native-controlled-stack"])
        .arg(binary)
        .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1")
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .status()
        .unwrap()
}

#[test]
fn rejected_or_ineffective_stack_limits_never_execute_the_native_probe() {
    for (override_limit, code) in [
        ("ulimit() { return 1; }; ", 125),
        (
            "ulimit() { if [ \"$#\" = 2 ]; then return 0; fi; printf '8192\\n'; }; ",
            126,
        ),
    ] {
        let status = Command::new("bash")
            .args([
                "-c",
                &format!("{override_limit}{LAUNCH}"),
                "stack-failure-control",
                "/bin/true",
            ])
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(code));
    }
    assert!(run(Path::new("/bin/true")).success());
}
