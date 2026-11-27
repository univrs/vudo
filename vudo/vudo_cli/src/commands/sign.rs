//! `vudo sign` - Sign package with Ed25519 identity

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use ed25519_dalek::SigningKey;
use spirit_runtime::Manifest;
use std::fs;
use std::path::{Path, PathBuf};
use vudo_cli::node::{content_hash, SignedPackage};

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct SignArgs {
    /// Spirit to sign: a WASM module (`vudo build` output) or a `vudo pack`
    /// package. With `--verify`: a signed package.
    pub package: PathBuf,

    /// Path to signing key (defaults to ~/.config/vudo/keys/default.key on Linux/macOS, %APPDATA%\\vudo\\keys\\default.key on Windows)
    #[arg(short, long)]
    pub key: Option<PathBuf>,

    /// Manifest for a WASM module (defaults to manifest.toml next to it).
    /// Its `author` must be the signing key's public key.
    #[arg(short, long)]
    pub manifest: Option<PathBuf>,

    /// Where to write the signed package (defaults to <input>.signed.spirit)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Verify signature instead of signing
    #[arg(long)]
    pub verify: bool,
}

pub async fn execute(args: SignArgs, config: &VudoConfig) -> Result<()> {
    if args.verify {
        verify_package(&args.package)?;
    } else {
        sign_package(&args, config)?;
    }

    Ok(())
}

/// Sign into a node package (Spirit Hosting H2d-3): JSON `{manifest, wasm,
/// wasm_signature}`, the format a DOLOS node publishes and runs.
fn sign_package(args: &SignArgs, config: &VudoConfig) -> Result<()> {
    let package_path = &args.package;
    println!(
        "{} Spirit package: {:?}",
        "Signing".green().bold(),
        package_path
    );

    let data = fs::read(package_path)
        .with_context(|| format!("Failed to read package: {:?}", package_path))?;
    let (manifest, wasm) = manifest_and_wasm(package_path, &data, args.manifest.as_ref())?;

    println!("  {} {} bytes", "WASM size:".cyan(), wasm.len());

    let key_file = args.key.clone().unwrap_or_else(|| default_key_path(config));
    let key = load_or_create_signing_key(&key_file)?;
    let key = spirit_runtime::SigningKey::from_bytes(&key.to_bytes())
        .map_err(|e| anyhow::anyhow!("signing key: {e}"))?;

    let signed = SignedPackage::sign(manifest, wasm, &key)?;
    let bytes = signed.to_bytes()?;

    println!(
        "  {} {}",
        "Public key:".cyan(),
        key.verifying_key().to_hex()
    );
    println!("  {} {}", "Name:".cyan(), signed.name());
    println!("  {} {}", "Content hash:".cyan(), content_hash(&bytes));

    let signed_path = args
        .output
        .clone()
        .unwrap_or_else(|| package_path.with_extension("signed.spirit"));
    fs::write(&signed_path, bytes)
        .with_context(|| format!("Failed to write signed package: {:?}", signed_path))?;

    println!("\n{} Signed package: {:?}", "✓".green().bold(), signed_path);

    Ok(())
}

/// The manifest and WASM to sign: from a `vudo pack` package, or a WASM
/// module plus its manifest.toml.
fn manifest_and_wasm<'a>(
    path: &Path,
    data: &'a [u8],
    manifest_path: Option<&PathBuf>,
) -> Result<(Manifest, &'a [u8])> {
    if let Some(rest) = data.strip_prefix(b"MANIFEST\n".as_slice()) {
        const SEP: &[u8] = b"\n\nWASM\n";
        let at = rest
            .windows(SEP.len())
            .position(|w| w == SEP)
            .context("Invalid vudo pack package: no WASM section")?;
        let toml_src = std::str::from_utf8(&rest[..at]).context("manifest is not UTF-8")?;
        let manifest = toml::from_str(toml_src).context("Failed to parse packaged manifest")?;
        return Ok((manifest, &rest[at + SEP.len()..]));
    }
    if !data.starts_with(b"\0asm") {
        anyhow::bail!("{path:?} is neither a WASM module nor a `vudo pack` package");
    }
    let manifest_file = manifest_path.cloned().unwrap_or_else(|| {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .join("manifest.toml")
    });
    let src = fs::read_to_string(&manifest_file).with_context(|| {
        format!(
            "Failed to read manifest {:?} (use --manifest)",
            manifest_file
        )
    })?;
    let manifest = toml::from_str(&src)
        .with_context(|| format!("Failed to parse manifest {:?}", manifest_file))?;
    Ok((manifest, data))
}

fn verify_package(package_path: &PathBuf) -> Result<()> {
    println!(
        "{} Spirit package: {:?}",
        "Verifying".green().bold(),
        package_path
    );

    let bytes = fs::read(package_path)
        .with_context(|| format!("Failed to read package: {:?}", package_path))?;
    let package = SignedPackage::from_bytes(&bytes)?;
    package.verify().context("Signature verification failed")?;

    println!("  {} {}", "Public key:".cyan(), package.manifest.author);
    println!("  {} {}", "Name:".cyan(), package.name());
    println!("  {} {}", "Content hash:".cyan(), content_hash(&bytes));
    println!("\n{} Signature is valid!", "✓".green().bold());

    Ok(())
}

/// The developer's default signing key: `<vudo dir>/keys/default.key`.
/// `vudo new` writes its public key as the manifest author; `vudo sign` signs with it.
pub(crate) fn default_key_path(config: &VudoConfig) -> PathBuf {
    config.vudo_dir().join("keys").join("default.key")
}

/// Load the signing key at `key_file`, generating and saving one if it does not exist.
pub(crate) fn load_or_create_signing_key(key_file: &PathBuf) -> Result<SigningKey> {
    if key_file.exists() {
        println!("  {} {:?}", "Using key:".cyan(), key_file);
        return load_signing_key(key_file);
    }
    println!(
        "  {} Generating new Ed25519 keypair",
        "Key not found:".yellow()
    );
    if let Some(parent) = key_file.parent() {
        fs::create_dir_all(parent)?;
        restrict_permissions(parent, 0o700)?;
    }
    let key = generate_signing_key();
    save_signing_key(&key, key_file)?;
    println!("  {} {:?}", "Saved key to:".green(), key_file);
    Ok(key)
}

fn generate_signing_key() -> SigningKey {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    let mut csprng = OsRng;
    let secret_bytes: [u8; 32] = rand::Rng::gen(&mut csprng);
    SigningKey::from_bytes(&secret_bytes)
}

fn save_signing_key(key: &SigningKey, path: &PathBuf) -> Result<()> {
    let key_bytes = key.to_bytes();
    write_private(path, hex::encode(key_bytes).as_bytes())
        .with_context(|| format!("Failed to save key to {:?}", path))
}

/// Write a private-key file readable only by its owner (0600 on Unix). The
/// mode is set at creation, so the key is never briefly world-readable.
fn write_private(path: &PathBuf, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)?.write_all(contents)
}

/// Restrict a directory to its owner on Unix; no-op elsewhere.
fn restrict_permissions(path: &std::path::Path, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .with_context(|| format!("Failed to restrict permissions on {:?}", path))?;
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

fn load_signing_key(path: &PathBuf) -> Result<SigningKey> {
    let key_hex =
        fs::read_to_string(path).with_context(|| format!("Failed to read key from {:?}", path))?;

    let key_bytes = hex::decode(key_hex.trim()).context("Invalid key format (expected hex)")?;

    let key_array: [u8; 32] = key_bytes
        .as_slice()
        .try_into()
        .context("Invalid key length (expected 32 bytes)")?;

    Ok(SigningKey::from_bytes(&key_array))
}
