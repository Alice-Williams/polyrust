//! External native test scaffolding, anchored to original certified definitions.
use crate::ast::{JavaFileItem, JavaMember, JavaModifier};
use crate::dialect::{JavaDependencyApi, JavaDependencyType};

fn spelling(ty: &JavaDependencyType) -> String {
    match ty {
        JavaDependencyType::Primitive(value) => value.keyword().to_owned(),
        JavaDependencyType::Result(value) => value.path().text(),
    }
}

pub(super) fn instrument(text: &str, api: &JavaDependencyApi) -> String {
    let files = api.package().ast().files();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].items().len(), 1);
    let JavaFileItem::Type { declaration, .. } = &files[0].items()[0].item else {
        panic!("certified producer facade")
    };
    let mut text = text.to_owned();
    for (name, event) in [("select", 1), ("successArm", 2), ("errorArm", 3)] {
        let function = api
            .functions()
            .find(|function| function.path().member().as_str() == name)
            .unwrap();
        let path = function.path();
        assert_eq!(path.package(), files[0].module().to_owned());
        assert_eq!(path.owners(), std::slice::from_ref(&declaration.name));
        assert_eq!(
            text.lines()
                .filter(|line| *line == format!("package {};", path.package().name()))
                .count(),
            1
        );
        assert_eq!(
            text.lines()
                .filter(
                    |line| *line == format!("public final class {} {{", declaration.name.as_str())
                )
                .count(),
            1
        );
        let methods = declaration
            .members
            .iter()
            .filter_map(|member| match member {
                JavaMember::Method(method) if method.name == *path.member() => Some(method),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(methods.len(), 1);
        let method = methods[0];
        assert_eq!(
            method.modifiers,
            [JavaModifier::Public, JavaModifier::Static]
        );
        assert!(method.type_parameters.is_empty() && method.annotations.is_empty());
        assert_eq!(method.return_type, function.declaration_signature().result);
        let signature = function.exported_signature();
        assert_eq!(method.parameters.len(), signature.parameters().len());
        let parameters = method
            .parameters
            .iter()
            .zip(signature.parameters())
            .enumerate()
            .map(|(index, (parameter, ty))| {
                assert_eq!(
                    parameter.ty,
                    function.declaration_signature().parameters[index]
                );
                assert!(parameter.final_parameter);
                format!("final {} {}", spelling(ty), parameter.name.as_str())
            })
            .collect::<Vec<_>>()
            .join(", ");
        let header = format!(
            "public static {} {}({parameters}) {{",
            spelling(signature.result()),
            path.member().as_str()
        );
        let definitions = text
            .lines()
            .filter(|line| line.trim_start() == header)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(
            definitions.len(),
            1,
            "observer needs one certified definition: {name}"
        );
        let line = &definitions[0];
        let original = format!("{line}\n");
        assert_eq!(text.matches(&original).count(), 1);
        text = text.replacen(
            &original,
            &format!("{line}\n        polyrust.test.TraceObserver.event({event});\n"),
            1,
        );
    }
    text
}

#[test]
fn observer_rejects_owner_and_parameter_header_substitutions() {
    let packages = super::fixture::packages(super::fixture::Mutation::None);
    let text = super::text(packages.producer.package());
    instrument(&text, &packages.producer);
    for (original, replacement) in [
        ("final boolean p0", "final Boolean p0"),
        ("final int p1", "final long p1"),
        ("public final class Generated", "public final class Other"),
        (
            "package org.polyrust.generated.r0000000000000009;",
            "package wrong;",
        ),
    ] {
        let altered = text.replacen(original, replacement, 1);
        assert_ne!(altered, text);
        assert!(std::panic::catch_unwind(|| instrument(&altered, &packages.producer)).is_err());
    }
}

pub(super) const OBSERVER: &str = r#"
package polyrust.test;
public final class TraceObserver {
    private TraceObserver() {}
    private static final int[] EVENTS = new int[8];
    private static int count;
    public static void reset() { count = 0; }
    public static void event(int value) {
        if (count < EVENTS.length) EVENTS[count] = value;
        if (count < EVENTS.length + 1) count++;
    }
    public static boolean selected(boolean success) {
        return count == 2 && EVENTS[0] == 1 && EVENTS[1] == (success ? 2 : 3);
    }
}
"#;

pub(super) const DRIVER: &str = r#"
public final class Consumer {
    private static int observations;
    private static int valueFailures;
    private static int traceFailures;
    private Consumer() {}
    private static void check(int value, boolean success, boolean trace) {
        org.polyrust.generated.r0000000000000007.Generated.Outcome nominal =
            org.polyrust.generated.r0000000000000009.Generated.select(success, value);
        boolean nominalValid = success
            ? nominal instanceof org.polyrust.generated.r0000000000000007.Generated.Success payload && payload.value() == value
            : nominal instanceof org.polyrust.generated.r0000000000000007.Generated.Error;
        polyrust.test.TraceObserver.reset();
        int actual = org.polyrust.generated.r000000000000000a.Generated.entry(success, value);
        if (!nominalValid || actual != (success ? value : 17)) valueFailures++;
        if (trace && !polyrust.test.TraceObserver.selected(success)) traceFailures++;
        observations++;
    }
    public static void main(String[] args) {
        if (args.length != 1 || !(args[0].equals("true") || args[0].equals("false"))) {
            throw new AssertionError("trace mode");
        }
        boolean trace = Boolean.parseBoolean(args[0]);
        for (int value = -256; value < 256; value++) {
            check(value, false, trace);
            check(value, true, trace);
        }
        for (int value : new int[] {Integer.MIN_VALUE, Integer.MAX_VALUE, -1, 0, 1, 17}) {
            check(value, false, trace);
            check(value, true, trace);
        }
        System.out.println(observations + " observations; " + valueFailures
            + " value failures; " + traceFailures + " trace failures");
    }
}
"#;
