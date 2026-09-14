use crate::{Owner, PreparedBundle, fixture, projection};

#[test]
fn manifest_mutations_never_reconstruct_owner_authority() {
    let api = fixture::owner(7, " Documentation.");
    let other = fixture::owner(7, " Different documentation.");
    let owner = Owner {
        key: "fixture",
        api: &api,
    };
    let mut manifest = projection::project(owner).unwrap();
    manifest.verify_owner().unwrap();
    let saved = manifest.declarations.clone();
    manifest.declarations.clear();
    assert!(manifest.verify_owner().is_err());
    manifest.declarations = saved.clone();
    manifest.declarations.push(saved[0]);
    assert!(manifest.verify_owner().is_err());
    manifest.declarations = other.source_descriptions().unwrap();
    assert!(manifest.verify_owner().is_err());
    manifest.declarations = saved;
    let root = api.root();
    let module = manifest.modules.remove(&root).unwrap();
    assert!(manifest.verify_owner().is_err());
    manifest.modules.insert(root, module);
    manifest.filename = "wrong.api.json".into();
    assert!(manifest.verify_owner().is_err());
}

#[test]
fn owner_limits_unused_members_and_order_are_exact() {
    let apis: Vec<_> = (1..=1025).map(|id| fixture::owner(id, "")).collect();
    let keys: Vec<_> = (1..=1025).map(|id| format!("owner.{id}")).collect();
    let mut owners: Vec<_> = keys
        .iter()
        .zip(&apis)
        .map(|(key, api)| Owner { key, api })
        .collect();
    let root = apis[0].root();
    assert!(PreparedBundle::new(root, &owners).is_err());
    assert!(PreparedBundle::new(root, &[]).is_err());
    owners.pop();
    let prepared = PreparedBundle::new(root, &owners).unwrap();
    let output = prepared.render().unwrap();
    assert_eq!(output.owner_count(), 1024);
    assert_eq!(output.files().len(), 2049);
    let paths: std::collections::BTreeSet<_> =
        output.files().iter().map(|(name, _)| name).collect();
    assert_eq!(paths.len(), 2049);
    let mut directories = std::collections::BTreeSet::new();
    for path in &paths {
        let mut parent = std::path::Path::new(path).parent();
        while let Some(value) = parent.filter(|value| !value.as_os_str().is_empty()) {
            assert!(value.components().count() <= 7);
            directories.insert(value.to_owned());
            parent = value.parent();
        }
    }
    assert_eq!(directories.len(), 1030);
    owners.reverse();
    assert_eq!(
        output.files(),
        PreparedBundle::new(root, &owners)
            .unwrap()
            .render()
            .unwrap()
            .files()
    );
    owners[0].key = owners[1].key;
    assert!(PreparedBundle::new(root, &owners).is_err());
    assert!(PreparedBundle::new(apis[1024].root(), &owners[1..]).is_err());
}

#[test]
fn metadata_escaping_and_repeated_fields_are_reserved_before_encoding() {
    let api = fixture::owner(7, " Quotes \" and \\ and café 🦀.\n Second line.");
    let owners = [Owner {
        key: "key\"\\\0\n🦀",
        api: &api,
    }];
    let prepared = PreparedBundle::new(api.root(), &owners).unwrap();
    let output = prepared.render().unwrap();
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
        .find(|(name, _)| name.ends_with(".api.json"))
        .unwrap()
        .1;
    assert!(json.contains("key\\\"\\\\\\u0000\\u000a🦀"));
    assert_eq!(json.matches("Quotes").count(), 2);
    assert!(json.ends_with('\n'));
    assert_eq!(json.matches('\n').count(), 1);
}
