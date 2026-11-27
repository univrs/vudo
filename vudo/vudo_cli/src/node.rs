//! A DOLOS node's Spirit package format and HTTP API (Spirit Hosting Plan H2d-3).
//!
//! This is the client side of univrs/network `mycelial-node` `spirit_host`:
//!
//! * **Package**: JSON `{manifest, wasm, wasm_signature}`. `manifest` is a
//!   [`Manifest`] signed by its `author` key (`Manifest::sign`), `wasm` is the
//!   module in standard base64, and `wasm_signature` is the author's hex
//!   Ed25519 signature over SHA-256(wasm) (`SigningKey::sign_prehashed`). A
//!   node refuses (403) a package unless both signatures verify.
//! * **Publish**: `POST /api/packages` with the exact package bytes. The node
//!   stores them under SHA-256(bytes), announces them, and puts the name
//!   record `author/name@version` → hash into the DHT.
//! * **Summon**: `POST /api/spirits {name | hash, entry, args, fuel}`. The node
//!   resolves the name in the DHT, fetches the package from a provider through
//!   its verified path, and runs it. `201` = the run completed; `422` = it ran
//!   and failed.

use anyhow::{bail, Context, Result};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use spirit_runtime::{Manifest, Signature, SigningKey, VerifyingKey};

/// A signed Spirit package exactly as a node stores and serves it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedPackage {
    pub manifest: Manifest,
    /// Base64 (standard) WASM module.
    pub wasm: String,
    /// Hex Ed25519 signature by the manifest author over SHA-256(wasm).
    pub wasm_signature: String,
}

impl SignedPackage {
    /// Sign `wasm` and `manifest` with `key`. The manifest's `author` must be
    /// `key`'s public key: a node refuses anything else, and the author is
    /// part of the Spirit's name, so it is never rewritten here.
    pub fn sign(mut manifest: Manifest, wasm: &[u8], key: &SigningKey) -> Result<Self> {
        check_wasm(wasm)?;
        let public = key.verifying_key().to_hex();
        if !manifest.author.eq_ignore_ascii_case(&public) {
            bail!(
                "manifest author {} is not the signing key {public}; set `author = \"{public}\"` in the manifest or sign with the author's key",
                manifest.author
            );
        }
        manifest.signature = None;
        let ed = ed25519_dalek::SigningKey::from_bytes(&key.to_bytes());
        manifest.signature = Some(manifest.sign(&ed).context("sign manifest")?);
        Ok(Self {
            manifest,
            wasm: base64::engine::general_purpose::STANDARD.encode(wasm),
            wasm_signature: key.sign_prehashed(wasm).to_hex(),
        })
    }

    /// The checks a node makes before it stores or loads a package. Returns
    /// the WASM bytes.
    pub fn verify(&self) -> Result<Vec<u8>> {
        self.manifest
            .verify()
            .map_err(|e| anyhow::anyhow!("manifest: {e}"))?;
        let wasm = base64::engine::general_purpose::STANDARD
            .decode(&self.wasm)
            .context("wasm is not base64")?;
        check_wasm(&wasm)?;
        let author = VerifyingKey::from_hex(&self.manifest.author)
            .map_err(|e| anyhow::anyhow!("author key: {e}"))?;
        let sig = Signature::from_hex(&self.wasm_signature)
            .map_err(|e| anyhow::anyhow!("wasm_signature: {e}"))?;
        author
            .verify_prehashed(&wasm, &sig)
            .map_err(|_| anyhow::anyhow!("wasm is not signed by the manifest author"))?;
        Ok(wasm)
    }

    /// The Spirit's network name, `author/name@version`.
    pub fn name(&self) -> String {
        let m = &self.manifest;
        format!("{}/{}@{}", m.author.to_ascii_lowercase(), m.name, m.version)
    }

    /// The bytes to publish. Their SHA-256 is the package's content address.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec_pretty(self)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).context("not a signed Spirit package (run `vudo sign`)")
    }
}

fn check_wasm(wasm: &[u8]) -> Result<()> {
    if wasm.len() < 8 || &wasm[0..4] != b"\0asm" {
        bail!("not a WASM module (missing \\0asm magic)");
    }
    Ok(())
}

/// Content address: lowercase hex SHA-256 of the exact package bytes.
pub fn content_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn is_content_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// What a node answers to a publish.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Published {
    pub hash: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub size: usize,
}

impl Published {
    /// `author/name@version`, the name to summon it by.
    pub fn spirit_name(&self) -> String {
        format!(
            "{}/{}@{}",
            self.author.to_ascii_lowercase(),
            self.name,
            self.version
        )
    }
}

/// A Spirit instance as a node reports it after a run.
#[derive(Debug, Clone, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub entry: String,
    /// `completed` or `failed`.
    pub state: String,
    pub result: Vec<Value>,
    pub fuel_consumed: u64,
    #[serde(default)]
    pub credits_charged: u64,
    pub error: Option<String>,
}

/// Which Spirit to summon: a name `author/name@version` or a content hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Name(String),
    Hash(String),
}

impl std::str::FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if is_content_hash(s) {
            return Ok(Target::Hash(s.to_string()));
        }
        let ok = s
            .split_once('/')
            .and_then(|(author, rest)| rest.rsplit_once('@').map(|(n, v)| (author, n, v)))
            .is_some_and(|(author, name, version)| {
                author.len() == 64
                    && author.bytes().all(|b| b.is_ascii_hexdigit())
                    && !name.is_empty()
                    && !name.contains('/')
                    && !version.is_empty()
            });
        if !ok {
            bail!("not a Spirit name `<author key hex>/<name>@<version>` or a SHA-256 hash: {s}");
        }
        Ok(Target::Name(s.to_string()))
    }
}

/// A summon: run `entry(args)` of the target on the node.
#[derive(Debug, Clone)]
pub struct Summon {
    pub target: Target,
    pub entry: String,
    pub args: Vec<i64>,
    pub fuel: Option<u64>,
}

fn url(node: &str, path: &str) -> String {
    format!("{}{path}", node.trim_end_matches('/'))
}

fn api_error(status: reqwest::StatusCode, body: &str) -> anyhow::Error {
    let msg = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["error"].as_str().map(str::to_string))
        .unwrap_or_else(|| body.to_string());
    anyhow::anyhow!("node answered {status}: {msg}")
}

/// Publish package `bytes` to the node at `node` (base URL, e.g.
/// `http://127.0.0.1:8080`). The package is verified locally first; the
/// node's content address must equal SHA-256 of the bytes sent.
pub async fn publish(node: &str, bytes: &[u8]) -> Result<Published> {
    SignedPackage::from_bytes(bytes)?
        .verify()
        .context("refusing to publish")?;
    let resp = reqwest::Client::new()
        .post(url(node, "/api/packages"))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(bytes.to_vec())
        .send()
        .await
        .with_context(|| format!("cannot reach node {node}"))?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(api_error(status, &body));
    }
    let published: Published =
        serde_json::from_str(&body).with_context(|| format!("unexpected answer: {body}"))?;
    let hash = content_hash(bytes);
    if published.hash != hash {
        bail!(
            "node stored the package as {}, but its content hash is {hash}",
            published.hash
        );
    }
    Ok(published)
}

/// Summon a Spirit on the node at `node`: it is resolved, fetched and run
/// there. `Ok` only if the node ran it to completion.
pub async fn summon(node: &str, req: &Summon) -> Result<Instance> {
    let mut body = serde_json::json!({ "entry": req.entry, "args": req.args });
    match &req.target {
        Target::Name(n) => body["name"] = Value::from(n.as_str()),
        Target::Hash(h) => body["hash"] = Value::from(h.as_str()),
    }
    if let Some(f) = req.fuel {
        body["fuel"] = Value::from(f);
    }
    let resp = reqwest::Client::new()
        .post(url(node, "/api/spirits"))
        .json(&body)
        .send()
        .await
        .with_context(|| format!("cannot reach node {node}"))?;
    let status = resp.status();
    let text = resp.text().await?;
    if status != reqwest::StatusCode::CREATED && status != reqwest::StatusCode::UNPROCESSABLE_ENTITY
    {
        return Err(api_error(status, &text));
    }
    let instance: Instance =
        serde_json::from_str(&text).with_context(|| format!("unexpected answer: {text}"))?;
    if status != reqwest::StatusCode::CREATED || instance.state != "completed" {
        bail!(
            "Spirit {} ran on the node and failed: {}",
            instance.id,
            instance.error.as_deref().unwrap_or("unknown error")
        );
    }
    Ok(instance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spirit_runtime::SemVer;

    const WASM: &[u8] = b"\0asm\x01\0\0\0";

    fn manifest_for(key: &SigningKey) -> Manifest {
        Manifest::new("hello", SemVer::new(0, 1, 0), key.verifying_key().to_hex())
    }

    #[test]
    fn signed_package_passes_the_node_checks() {
        let key = SigningKey::generate();
        let pkg = SignedPackage::sign(manifest_for(&key), WASM, &key).unwrap();
        assert_eq!(pkg.verify().unwrap(), WASM);
        // Exactly the node's checks (spirit_host::verify), spelled out.
        pkg.manifest.verify().unwrap();
        let wasm = base64::engine::general_purpose::STANDARD
            .decode(&pkg.wasm)
            .unwrap();
        VerifyingKey::from_hex(&pkg.manifest.author)
            .unwrap()
            .verify_prehashed(&wasm, &Signature::from_hex(&pkg.wasm_signature).unwrap())
            .unwrap();
        // Wire shape: exactly these three fields.
        let v: Value = serde_json::from_slice(&pkg.to_bytes().unwrap()).unwrap();
        let mut keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(keys, ["manifest", "wasm", "wasm_signature"]);
        assert_eq!(
            pkg.name(),
            format!("{}/hello@0.1.0", key.verifying_key().to_hex())
        );
    }

    #[test]
    fn sign_never_signs_for_another_author() {
        let key = SigningKey::generate();
        let other = SigningKey::generate();
        let err = SignedPackage::sign(manifest_for(&other), WASM, &key).unwrap_err();
        assert!(err.to_string().contains("not the signing key"), "{err}");
    }

    #[test]
    fn sign_refuses_non_wasm() {
        let key = SigningKey::generate();
        assert!(SignedPackage::sign(manifest_for(&key), b"MANIFEST\nx", &key).is_err());
    }

    #[test]
    fn tampered_wasm_or_manifest_fails_verification() {
        let key = SigningKey::generate();
        let pkg = SignedPackage::sign(manifest_for(&key), WASM, &key).unwrap();

        let mut wasm = pkg.clone();
        wasm.wasm = base64::engine::general_purpose::STANDARD.encode(b"\0asm\x01\0\0\x01");
        assert!(wasm.verify().is_err(), "swapped wasm must not verify");

        let mut manifest = pkg.clone();
        manifest.manifest.name = "evil".into();
        assert!(
            manifest.verify().is_err(),
            "edited manifest must not verify"
        );

        let mut unsigned = pkg;
        unsigned.manifest.signature = None;
        assert!(unsigned.verify().is_err());
    }

    #[test]
    fn targets_are_names_or_hashes() {
        let author = "ab".repeat(32);
        let hash = "0f".repeat(32);
        assert_eq!(
            format!("{author}/hello@0.1.0").parse::<Target>().unwrap(),
            Target::Name(format!("{author}/hello@0.1.0"))
        );
        assert_eq!(hash.parse::<Target>().unwrap(), Target::Hash(hash.clone()));
        for bad in [
            "hello",
            "@someone/thing",
            "abc/hello@1",
            &format!("{author}/hello"),
        ] {
            assert!(bad.parse::<Target>().is_err(), "{bad}");
        }
    }

    #[tokio::test]
    async fn publish_refuses_an_unsigned_package_before_any_network_call() {
        let key = SigningKey::generate();
        let mut pkg = SignedPackage::sign(manifest_for(&key), WASM, &key).unwrap();
        pkg.manifest.signature = None;
        // Port 9 (discard) on a reserved address: a network attempt would
        // fail with "cannot reach", not "refusing to publish".
        let err = publish("http://127.0.0.1:9", &pkg.to_bytes().unwrap())
            .await
            .unwrap_err();
        assert!(
            format!("{err:#}").contains("refusing to publish"),
            "{err:#}"
        );
    }
}
