//! Spirit Hosting Plan H1a: manifest `[resources]`.

use ed25519_dalek::SigningKey;
use spirit_runtime::{Manifest, Resources};

const BASE: &str = r#"name = "res-spirit"
version = { major = 0, minor = 1, patch = 0 }
author = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
description = "resources test"
"#;

#[test]
fn manifest_without_resources_still_parses() {
    // Negative oracle: pre-H1a manifests are never rejected.
    let m = Manifest::from_toml(BASE).expect("old manifest must parse");
    assert_eq!(m.resources, None);
    assert!(
        !m.to_toml().unwrap().contains("[resources]"),
        "no section emitted when absent"
    );
}

#[test]
fn manifest_resources_parse_and_round_trip() {
    let src =
        format!("{BASE}\n[resources]\nmemory_bytes = 1048576\nfuel = 50000\ncpu_hint = 0.25\n");
    let m = Manifest::from_toml(&src).expect("parse");
    let r = m.resources.clone().expect("resources");
    assert_eq!(
        r,
        Resources {
            memory_bytes: Some(1_048_576),
            fuel: Some(50_000),
            cpu_hint: Some(0.25)
        }
    );
    let back = Manifest::from_toml(&m.to_toml().unwrap()).expect("round trip");
    assert_eq!(back.resources, Some(r));
}

#[test]
fn partial_resources_leave_other_fields_unset() {
    let m = Manifest::from_toml(&format!("{BASE}\n[resources]\nfuel = 7\n")).unwrap();
    assert_eq!(
        m.resources,
        Some(Resources {
            fuel: Some(7),
            ..Default::default()
        })
    );
}

#[test]
fn signature_covers_declared_resources() {
    let key = SigningKey::from_bytes(&[9u8; 32]);
    let mut m = Manifest::from_toml(&format!("{BASE}\n[resources]\nfuel = 1000\n")).unwrap();
    m.author = hex::encode(key.verifying_key().to_bytes());
    let sig = m.sign(&key).unwrap();
    m.signature = Some(sig);
    m.verify().expect("signed manifest verifies");

    // Negative oracle: inflating a signed Spirit's fuel must break the signature.
    m.resources.as_mut().unwrap().fuel = Some(1_000_000_000);
    assert!(m.verify().is_err(), "tampered resources must never verify");
}

#[test]
fn absent_resources_do_not_change_the_signed_hash() {
    let m = Manifest::from_toml(BASE).unwrap();
    let mut with_none = m.clone();
    with_none.resources = None;
    assert_eq!(m.content_hash(), with_none.content_hash());
}
