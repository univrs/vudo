# Ed25519 Signing Guide

This guide covers the cryptographic signing workflow for Spirit packages using Ed25519 digital signatures.

## Overview

VUDO uses Ed25519 signatures to ensure:
- **Authenticity**: Verify the Spirit was created by the claimed author
- **Integrity**: Detect any tampering with the manifest content
- **Non-repudiation**: Authors cannot deny creating a signed Spirit

## Ed25519 Basics

Ed25519 is a high-performance elliptic curve signature algorithm that provides:
- 128-bit security level (equivalent to ~3000-bit RSA)
- 64-byte signatures
- 32-byte public keys
- 32-byte private keys
- Deterministic signatures (same input = same output)
- Fast signing and verification

## Key Management

### Generating Keys

```rust
use spirit_runtime::signature::{KeyPair, SigningKey};

// Generate a complete keypair
let keypair = KeyPair::generate();

// Or generate just a signing key
let signing_key = SigningKey::generate();
let verifying_key = signing_key.verifying_key();
```

### Key Formats

Keys can be represented in multiple formats:

```rust
use spirit_runtime::signature::SigningKey;

let signing_key = SigningKey::generate();

// Raw bytes (32 bytes)
let bytes: [u8; 32] = signing_key.to_bytes();
let restored = SigningKey::from_bytes(&bytes)?;

// Hex string (64 characters)
let hex: String = signing_key.to_hex();
let restored = SigningKey::from_hex(&hex)?;

// Slice (for variable-length input)
let slice: &[u8] = &bytes[..];
let restored = SigningKey::from_slice(slice)?;
```

### Key Storage

**Security Warning**: Signing keys are private keys. Handle them with care.

```rust
use spirit_runtime::signature::SigningKey;
use std::fs;

// Generate a new key
let signing_key = SigningKey::generate();

// Save to file (secure storage recommended)
let key_hex = signing_key.to_hex();
fs::write("signing_key.secret", &key_hex)?;

// Load from file
let key_hex = fs::read_to_string("signing_key.secret")?;
let signing_key = SigningKey::from_hex(&key_hex.trim())?;

// Get public key for sharing
let public_key = signing_key.verifying_key();
let public_hex = public_key.to_hex();
println!("Public key: {}", public_hex);
```

### Key Derivation

The verifying (public) key is derived from the signing (private) key:

```rust
use spirit_runtime::signature::SigningKey;

let signing_key = SigningKey::generate();
let verifying_key = signing_key.verifying_key();

// verifying_key can be freely shared
// signing_key must remain secret
```

## Signing Workflow

### Step 1: Generate or Load Your Key

```rust
use spirit_runtime::signature::{KeyPair, SigningKey};
use std::fs;
use std::path::Path;

fn get_or_create_keypair(path: &Path) -> Result<KeyPair, Box<dyn std::error::Error>> {
    if path.exists() {
        // Load existing key
        let hex = fs::read_to_string(path)?;
        let signing_key = SigningKey::from_hex(hex.trim())?;
        Ok(KeyPair::from_signing_key(signing_key))
    } else {
        // Generate new keypair
        let keypair = KeyPair::generate();
        let hex = keypair.signing_key().to_hex();
        fs::write(path, &hex)?;
        println!("Created new keypair at {:?}", path);
        println!("Public key: {}", keypair.verifying_key().to_hex());
        Ok(keypair)
    }
}
```

### Step 2: Create Your Manifest

```rust
use spirit_runtime::{ManifestBuilder, Capability, SemVer};

let keypair = get_or_create_keypair(Path::new("./signing_key.secret"))?;

let manifest = ManifestBuilder::new(
    "my-spirit",
    SemVer::new(1, 0, 0),
    keypair.verifying_key().to_hex(),  // Author = public key
)
.description("My signed Spirit")
.capability(Capability::SensorTime)
.build();
```

### Step 3: Sign the Manifest

```rust
use spirit_runtime::Manifest;

let mut manifest = /* your manifest */;

// Sign using the signing key
let signature = manifest.sign(&keypair.signing_key().0)?;

// Attach signature to manifest
manifest.signature = Some(signature);

// Save the signed manifest
manifest.to_file("manifest.toml")?;
```

### Step 4: Verify Before Distribution

```rust
use spirit_runtime::Manifest;

let manifest = Manifest::from_file("manifest.toml")?;

// Validate structure
manifest.validate()?;

// Verify signature
manifest.verify()?;

println!("Manifest is valid and properly signed");
```

## Complete Example

```rust
use spirit_runtime::{
    Manifest, ManifestBuilder, Capability, SemVer,
    signature::{KeyPair, SigningKey},
};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Key file path
    let key_path = Path::new("./my_spirit_key.secret");

    // Generate or load keypair
    let keypair = if key_path.exists() {
        let hex = fs::read_to_string(key_path)?;
        KeyPair::from_signing_key(SigningKey::from_hex(hex.trim())?)
    } else {
        let kp = KeyPair::generate();
        fs::write(key_path, kp.signing_key().to_hex())?;
        println!("New keypair generated");
        println!("Public key: {}", kp.verifying_key().to_hex());
        println!("Keep {} secure!", key_path.display());
        kp
    };

    // Create manifest with author = public key
    let mut manifest = ManifestBuilder::new(
        "secure-spirit",
        SemVer::new(1, 0, 0),
        keypair.verifying_key().to_hex(),
    )
    .description("A cryptographically signed Spirit")
    .license("MIT")
    .capability(Capability::SensorTime)
    .capability(Capability::ActuatorLog)
    .build();

    // Sign the manifest
    let signature = manifest.sign(&keypair.signing_key().0)?;
    manifest.signature = Some(signature.clone());

    println!("Manifest signed successfully");
    println!("Signature: {}...", &signature[..32]);

    // Validate and verify
    manifest.validate()?;
    manifest.verify()?;

    println!("Signature verified successfully");

    // Save signed manifest
    manifest.to_file("./my-spirit/manifest.toml")?;
    println!("Saved to ./my-spirit/manifest.toml");

    Ok(())
}
```

## Signature API Reference

### Signature Type

```rust
use spirit_runtime::signature::Signature;

// Length constant
assert_eq!(Signature::LENGTH, 64);

// Create from bytes
let sig = Signature::from_bytes(&[0u8; 64])?;
let sig = Signature::from_slice(&bytes[..])?;
let sig = Signature::from_hex("0123456789abcdef...")?;

// Convert to bytes/hex
let bytes: [u8; 64] = sig.to_bytes();
let hex: String = sig.to_hex();

// Serde serialization (as hex string)
let json = serde_json::to_string(&sig)?;
let sig: Signature = serde_json::from_str(&json)?;
```

### SigningKey Type

```rust
use spirit_runtime::signature::SigningKey;

// Generate
let key = SigningKey::generate();

// From bytes/hex
let key = SigningKey::from_bytes(&[0u8; 32])?;
let key = SigningKey::from_hex("0123...64chars")?;

// To bytes/hex
let bytes: [u8; 32] = key.to_bytes();
let hex: String = key.to_hex();

// Get public key
let verifying_key = key.verifying_key();

// Sign message
let signature = key.sign(b"message");

// Sign prehashed (SHA-256 first)
let signature = key.sign_prehashed(b"large message");
```

### VerifyingKey Type

```rust
use spirit_runtime::signature::VerifyingKey;

// From bytes/hex
let key = VerifyingKey::from_bytes(&[0u8; 32])?;
let key = VerifyingKey::from_hex("0123...64chars")?;

// To bytes/hex
let bytes: [u8; 32] = key.to_bytes();
let hex: String = key.to_hex();

// Verify signature
key.verify(b"message", &signature)?;

// Verify prehashed
key.verify_prehashed(b"large message", &signature)?;

// Serde serialization
let json = serde_json::to_string(&key)?;
let key: VerifyingKey = serde_json::from_str(&json)?;
```

### KeyPair Type

```rust
use spirit_runtime::signature::KeyPair;

// Generate
let keypair = KeyPair::generate();

// From signing key
let keypair = KeyPair::from_signing_key(signing_key);

// Access keys
let signing: &SigningKey = keypair.signing_key();
let verifying: VerifyingKey = keypair.verifying_key();

// Sign/verify
let signature = keypair.sign(b"message");
keypair.verify(b"message", &signature)?;
```

### Convenience Functions

```rust
use spirit_runtime::signature::{sign_with_new_key, verify};

// One-shot signing with new key
let (signature, public_key) = sign_with_new_key(b"message");

// One-shot verification
verify(b"message", &signature, &public_key)?;
```

## Error Handling

```rust
use spirit_runtime::signature::SignatureError;

match result {
    Ok(()) => println!("Success"),
    Err(SignatureError::VerificationFailed) => {
        println!("Signature does not match message/key");
    }
    Err(SignatureError::InvalidKey(msg)) => {
        println!("Malformed key: {}", msg);
    }
    Err(SignatureError::InvalidSignature(msg)) => {
        println!("Malformed signature: {}", msg);
    }
    Err(SignatureError::HexError(msg)) => {
        println!("Invalid hex encoding: {}", msg);
    }
}
```

## Security Best Practices

### Protect Private Keys

1. **Never commit private keys** to version control
2. **Use file permissions** to restrict access (e.g., `chmod 600`)
3. **Consider hardware security modules** for production
4. **Rotate keys periodically** and revoke compromised keys

### Verify Before Trust

```rust
// Always verify before using a Spirit
let manifest = Manifest::from_file("manifest.toml")?;

// Step 1: Structural validation
manifest.validate()?;

// Step 2: Cryptographic verification
manifest.verify()?;

// Step 3: Check author is trusted
let trusted_authors = vec!["abcd1234...", "5678efgh..."];
if !trusted_authors.contains(&manifest.author.as_str()) {
    return Err("Untrusted author".into());
}
```

### Content Hash

The signature covers a SHA-256 hash of the manifest content:

```rust
let manifest = Manifest::from_file("manifest.toml")?;
let hash = manifest.content_hash();

// Hash includes:
// - name
// - version (string format)
// - author
// - description (if present)
// - capabilities (debug format)

// Hash excludes:
// - signature field (allows signing after creation)
```

## Troubleshooting

### "Signature verification failed"

Possible causes:
1. Content was modified after signing
2. Wrong public key in manifest
3. Corrupted signature

```rust
// Debug: check content hash
let hash1 = original_manifest.content_hash();
let hash2 = loaded_manifest.content_hash();
if hash1 != hash2 {
    println!("Content was modified");
}
```

### "Invalid key format"

```rust
// Check key length
let hex = "...";
println!("Key length: {} chars", hex.len());
// Should be 64 for public key, 64 for private key

// Check hex validity
if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
    println!("Contains non-hex characters");
}
```

### "No signature present"

```rust
let manifest = Manifest::from_file("manifest.toml")?;
if manifest.signature.is_none() {
    println!("Manifest needs to be signed first");
}
```

## Next Steps

- [Manifest Guide](./manifest-guide.md) - Complete manifest format
- [Registry Guide](./registry-guide.md) - Publishing signed Spirits
