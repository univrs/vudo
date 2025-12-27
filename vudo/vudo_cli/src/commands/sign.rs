//! `vudo sign` - Sign package with Ed25519 identity

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct SignArgs {
    /// Path to Spirit package to sign
    pub package: PathBuf,

    /// Path to signing key (defaults to ~/.vudo/keys/default.key)
    #[arg(short, long)]
    pub key: Option<PathBuf>,

    /// Verify signature instead of signing
    #[arg(long)]
    pub verify: bool,
}

pub async fn execute(args: SignArgs, config: &VudoConfig) -> Result<()> {
    if args.verify {
        verify_package(&args.package)?;
    } else {
        sign_package(&args.package, args.key, config)?;
    }

    Ok(())
}

fn sign_package(
    package_path: &PathBuf,
    key_path: Option<PathBuf>,
    config: &VudoConfig,
) -> Result<()> {
    println!(
        "{} Spirit package: {:?}",
        "Signing".green().bold(),
        package_path
    );

    // Read package data
    let package_data = fs::read(package_path)
        .with_context(|| format!("Failed to read package: {:?}", package_path))?;

    println!("  {} {} bytes", "Package size:".cyan(), package_data.len());

    // Determine key path
    let key_file = key_path.unwrap_or_else(|| {
        let vudo_dir = config.vudo_dir();
        vudo_dir.join("keys").join("default.key")
    });

    // Load or create signing key
    let signing_key = if key_file.exists() {
        println!("  {} {:?}", "Using key:".cyan(), key_file);
        load_signing_key(&key_file)?
    } else {
        println!(
            "  {} Generating new Ed25519 keypair",
            "Key not found:".yellow()
        );

        // Create keys directory
        if let Some(parent) = key_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let key = generate_signing_key();
        save_signing_key(&key, &key_file)?;

        println!("  {} {:?}", "Saved key to:".green(), key_file);
        key
    };

    // Get public key
    let verifying_key = signing_key.verifying_key();
    let public_key_hex = hex::encode(verifying_key.as_bytes());

    println!("  {} {}", "Public key:".cyan(), public_key_hex);

    // Hash the package data
    let mut hasher = Sha256::new();
    hasher.update(&package_data);
    let hash = hasher.finalize();

    println!("  {} {}", "Package hash:".cyan(), hex::encode(&hash));

    // Sign the hash
    let signature = signing_key.sign(&hash);
    let signature_bytes = signature.to_bytes();

    println!(
        "  {} {}",
        "Signature:".cyan(),
        hex::encode(&signature_bytes)
    );

    // Create signed package
    let mut signed_package = Vec::new();
    signed_package.extend_from_slice(b"SIGNED\n");
    signed_package.extend_from_slice(verifying_key.as_bytes());
    signed_package.extend_from_slice(b"\n");
    signed_package.extend_from_slice(&signature_bytes);
    signed_package.extend_from_slice(b"\n");
    signed_package.extend_from_slice(&package_data);

    // Write signed package
    let signed_path = package_path.with_extension("signed.spirit");
    fs::write(&signed_path, signed_package)
        .with_context(|| format!("Failed to write signed package: {:?}", signed_path))?;

    println!("\n{} Signed package: {:?}", "✓".green().bold(), signed_path);

    Ok(())
}

fn verify_package(package_path: &PathBuf) -> Result<()> {
    println!(
        "{} Spirit package: {:?}",
        "Verifying".green().bold(),
        package_path
    );

    // Read package
    let package_data = fs::read(package_path)
        .with_context(|| format!("Failed to read package: {:?}", package_path))?;

    // Check if it's signed
    if !package_data.starts_with(b"SIGNED\n") {
        anyhow::bail!("Package is not signed");
    }

    // Parse signature components
    let content = String::from_utf8_lossy(&package_data);
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 4 {
        anyhow::bail!("Invalid signed package format");
    }

    let public_key_bytes = hex::decode(lines[1].trim()).context("Invalid public key format")?;
    let signature_bytes = hex::decode(lines[2].trim()).context("Invalid signature format")?;

    // Find where the actual package data starts
    let data_start = package_data
        .iter()
        .position(|&b| b == b'\n')
        .and_then(|i| {
            package_data[i + 1..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|j| i + j + 2)
        })
        .and_then(|i| {
            package_data[i + 1..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|j| i + j + 2)
        })
        .context("Invalid package format")?;

    let original_data = &package_data[data_start..];

    // Reconstruct verifying key
    let verifying_key = VerifyingKey::from_bytes(
        public_key_bytes
            .as_slice()
            .try_into()
            .context("Invalid public key length")?,
    )
    .context("Invalid public key")?;

    // Reconstruct signature
    let signature = Signature::from_bytes(
        signature_bytes
            .as_slice()
            .try_into()
            .context("Invalid signature length")?,
    );

    // Hash the original data
    let mut hasher = Sha256::new();
    hasher.update(original_data);
    let hash = hasher.finalize();

    // Verify signature
    verifying_key
        .verify(&hash, &signature)
        .context("Signature verification failed")?;

    println!(
        "  {} {}",
        "Public key:".cyan(),
        hex::encode(verifying_key.as_bytes())
    );
    println!("  {} {}", "Signature:".cyan(), hex::encode(signature_bytes));
    println!("\n{} Signature is valid!", "✓".green().bold());

    Ok(())
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
    fs::write(path, hex::encode(key_bytes))
        .with_context(|| format!("Failed to save key to {:?}", path))
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
