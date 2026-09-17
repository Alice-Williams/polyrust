//! Alias schema is reconstructed from the exact package and producer certificates.
use crate::{Owner, PreparedBundle, alias_fixture as a, constant_fixture as c, projection};
use portable_backend_java::ast::JavaLiteral;

#[test]
fn aliases_have_versioned_original_paths_without_inventing_owned_declarations_or_reads() {
    let producer = c::owner(7, "Producer.", JavaLiteral::I64(9_007_199_254_740_993));
    let second = c::owner(8, "Other.", JavaLiteral::Boolean(true));
    let values = [
        producer.constants().next().unwrap().clone(),
        second.constants().next().unwrap().clone(),
    ];
    let middle = a::owner(9, &values, false);
    let forwarded: Vec<_> = middle
        .foreign_constants()
        .filter(|v| v.module() == middle.root())
        .map(|v| v.dependency().clone())
        .collect();
    for read in [false, true] {
        let root = a::owner(10, &forwarded, read);
        let owners = [
            Owner {
                key: "producer",
                api: &producer,
            },
            Owner {
                key: "second",
                api: &second,
            },
            Owner {
                key: "middle",
                api: &middle,
            },
            Owner {
                key: "root",
                api: &root,
            },
        ];
        let prepared = PreparedBundle::new(root.root(), &owners).unwrap();
        let output = prepared.render().unwrap();
        assert_eq!(output.files().len(), 9);
        assert!(
            output
                .files()
                .iter()
                .map(|(_, value)| value.len() as u64)
                .sum::<u64>()
                <= prepared.reserved_bytes()
        );
        let json = &output
            .files()
            .iter()
            .find(|(p, _)| p == "polyrust_000000000000000a.api.json")
            .unwrap()
            .1;
        assert!(json.starts_with("{\"schema_version\":4,"));
        let mut exact = Vec::new();
        for module in ["0000000000000001", "0000000000000002"] {
            for (alias, producer, scalar, value) in [
                ("alias0", "0000000000000007", "i64", "\"9007199254740993\""),
                ("alias1", "0000000000000008", "bool", "true"),
            ] {
                exact.push(format!(
                    "{{\"module\":\"000000000000000a:{module}\",\"namespace\":\"value\",\"name\":\"{alias}\",\"id\":\"{producer}:0000000000000002\",\"owner\":\"{producer}:0000000000000001\",\"path\":{{\"package\":\"org.polyrust.generated.r{producer}\",\"owners\":[\"Generated\"],\"member\":\"VALUE\"}},\"scalar\":\"{scalar}\",\"readonly\":true,\"value\":{value}}}"
                ));
            }
        }
        assert_eq!(
            json.split_once(",\"constant_exports\":[").unwrap().1,
            format!("{}]}}\n", exact.join(","))
        );
        assert_eq!(
            json.matches("\"namespace\":\"value\",\"name\":\"alias")
                .count(),
            8
        ); // Graph + evidence.
        assert!(json.contains("\"owner\":\"0000000000000007:0000000000000001\""));
        assert!(json.contains("\"value\":\"9007199254740993\""));
        assert!(json.contains("\"value\":true"));
        assert_eq!(json.contains("\"constant_imports\""), read);
        assert_eq!(root.source_descriptions().unwrap().len(), usize::from(read));
        if !read {
            assert!(json.contains("\"declarations\":[]"));
        }
        let imports = crate::constant_imports::collect(&root).unwrap();
        assert_eq!(imports.len(), usize::from(read));
        let old = &output
            .files()
            .iter()
            .find(|(p, _)| p == "polyrust_0000000000000007.api.json")
            .unwrap()
            .1;
        assert!(old.starts_with("{\"schema_version\":2,"));
        let reverse: Vec<_> = owners.iter().rev().copied().collect();
        assert_eq!(
            output.files(),
            PreparedBundle::new(root.root(), &reverse)
                .unwrap()
                .render()
                .unwrap()
                .files()
        );
    }
}

#[test]
fn missing_or_recertified_alias_producers_never_pass_preflight() {
    let producer = c::owner(7, "", JavaLiteral::I32(42));
    let root = a::owner(9, &[producer.constants().next().unwrap().clone()], false);
    let root_owner = Owner {
        key: "root",
        api: &root,
    };
    assert!(PreparedBundle::new(root.root(), &[root_owner]).is_err());
    let same_looking = c::owner(7, "", JavaLiteral::I32(42));
    assert!(
        PreparedBundle::new(
            root.root(),
            &[
                root_owner,
                Owner {
                    key: "producer",
                    api: &same_looking
                }
            ]
        )
        .is_err()
    );
    PreparedBundle::new(
        root.root(),
        &[
            root_owner,
            Owner {
                key: "producer",
                api: &producer,
            },
        ],
    )
    .unwrap()
    .render()
    .unwrap();
}

#[test]
fn alias_projection_and_reservation_mutations_reject() {
    let producer = c::owner(7, "", JavaLiteral::I32(42));
    let value = producer.constants().next().unwrap().clone();
    let root = a::owner(9, std::slice::from_ref(&value), false);
    let other = a::owner(10, std::slice::from_ref(&value), false);
    let owner = Owner {
        key: "root",
        api: &root,
    };
    for fault in 0..5 {
        let mut manifest = projection::project(owner).unwrap();
        match fault {
            0 => manifest.foreign_constants.clear(),
            1 => manifest
                .foreign_constants
                .push(manifest.foreign_constants[0]),
            2 => manifest.foreign_constants.reverse(),
            3 => manifest.foreign_constants = other.foreign_constants().collect(),
            4 => {
                let replacement = projection::project(Owner {
                    key: "other",
                    api: &other,
                })
                .unwrap();
                manifest.foreign_constants = replacement.foreign_constants;
                manifest.exports = replacement.exports;
                manifest.modules = replacement.modules;
            }
            _ => unreachable!(),
        }
        assert!(manifest.verify_owner().is_err(), "fault {fault}");
    }
    let manifest = projection::project(owner).unwrap();
    let mut reservation = crate::json::Reservation::default();
    crate::serialization::owner(&mut reservation, &manifest).unwrap();
    let mut insufficient = crate::json::Encoder::new(1);
    assert!(crate::serialization::owner(&mut insufficient, &manifest).is_err());
    let mut enough = crate::json::Encoder::new(reservation.0.0);
    crate::serialization::owner(&mut enough, &manifest).unwrap();
}
