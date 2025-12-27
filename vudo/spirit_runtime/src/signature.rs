//! Ed25519 signature signing and verification for Spirit packages.
//!
//! This module provides cryptographic signing and verification using the Ed25519
//! digital signature algorithm, ensuring authenticity and integrity of Spirit packages.
//!
//! # Overview
//!
//! Ed25519 is a high-security, high-performance signature algorithm that provides:
//! - 128-bit security level
//! - Compact signatures (64 bytes)
//! - Fast signing and verification
//! - Deterministic signatures (same message + key = same signature)
//!
//! # Example
//!
//! ```
//! use spirit_runtime::signature::{SigningKey, VerifyingKey};
//!
//! // Generate a new keypair
//! let signing_key = SigningKey::generate();
//! let verifying_key = signing_key.verifying_key();
//!
//! // Sign a message
//! let message = b"Hello, Spirit!";
//! let signature = signing_key.sign(message);
//!
//! // Verify the signature
//! assert!(verifying_key.verify(message, &signature).is_ok());
//! ```

use ed25519_dalek::{
    Signature as DalekSignature, Signer, SigningKey as DalekSigningKey, Verifier,
    VerifyingKey as DalekVerifyingKey, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Errors that can occur during signature operations.
#[derive(Debug, Error)]
pub enum SignatureError {
    /// The signature is invalid or verification failed.
    #[error("signature verification failed")]
    VerificationFailed,

    /// The key bytes are malformed or invalid.
    #[error("invalid key format: {0}")]
    InvalidKey(String),

    /// The signature bytes are malformed or invalid.
    #[error("invalid signature format: {0}")]
    InvalidSignature(String),

    /// Error during hex encoding/decoding.
    #[error("hex encoding error: {0}")]
    HexError(String),
}

/// A 64-byte Ed25519 signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signature(DalekSignature);

impl Signature {
    /// The length of a signature in bytes.
    pub const LENGTH: usize = SIGNATURE_LENGTH;

    /// Creates a signature from raw bytes.
    pub fn from_bytes(bytes: &[u8; Self::LENGTH]) -> Result<Self, SignatureError> {
        Ok(Signature(DalekSignature::from_bytes(bytes)))
    }

    /// Creates a signature from a byte slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, SignatureError> {
        if bytes.len() != Self::LENGTH {
            return Err(SignatureError::InvalidSignature(format!(
                "expected {} bytes, got {}",
                Self::LENGTH,
                bytes.len()
            )));
        }
        let mut arr = [0u8; Self::LENGTH];
        arr.copy_from_slice(bytes);
        Self::from_bytes(&arr)
    }

    /// Creates a signature from a hex-encoded string.
    pub fn from_hex(hex: &str) -> Result<Self, SignatureError> {
        let bytes = hex::decode(hex).map_err(|e| SignatureError::HexError(e.to_string()))?;
        Self::from_slice(&bytes)
    }

    /// Returns the signature as raw bytes.
    pub fn to_bytes(&self) -> [u8; Self::LENGTH] {
        self.0.to_bytes()
    }

    /// Returns the signature as a hex-encoded string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        Signature::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

/// An Ed25519 signing key (private key).
///
/// This key is used to create signatures. Keep it secret!
pub struct SigningKey(DalekSigningKey);

impl SigningKey {
    /// The length of a signing key in bytes.
    pub const LENGTH: usize = SECRET_KEY_LENGTH;

    /// Generates a new random signing key using the OS random number generator.
    pub fn generate() -> Self {
        SigningKey(DalekSigningKey::generate(&mut OsRng))
    }

    /// Creates a signing key from raw bytes.
    pub fn from_bytes(bytes: &[u8; Self::LENGTH]) -> Result<Self, SignatureError> {
        Ok(SigningKey(DalekSigningKey::from_bytes(bytes)))
    }

    /// Creates a signing key from a byte slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, SignatureError> {
        if bytes.len() != Self::LENGTH {
            return Err(SignatureError::InvalidKey(format!(
                "expected {} bytes, got {}",
                Self::LENGTH,
                bytes.len()
            )));
        }
        let mut arr = [0u8; Self::LENGTH];
        arr.copy_from_slice(bytes);
        Self::from_bytes(&arr)
    }

    /// Creates a signing key from a hex-encoded string.
    pub fn from_hex(hex: &str) -> Result<Self, SignatureError> {
        let bytes = hex::decode(hex).map_err(|e| SignatureError::HexError(e.to_string()))?;
        Self::from_slice(&bytes)
    }

    /// Returns the signing key as raw bytes.
    ///
    /// # Security Warning
    ///
    /// Handle these bytes with care - they represent the private key.
    pub fn to_bytes(&self) -> [u8; Self::LENGTH] {
        self.0.to_bytes()
    }

    /// Returns the signing key as a hex-encoded string.
    ///
    /// # Security Warning
    ///
    /// Handle this string with care - it represents the private key.
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    /// Returns the corresponding verifying (public) key.
    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey(self.0.verifying_key())
    }

    /// Signs a message, returning the signature.
    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature(self.0.sign(message))
    }

    /// Signs a message after hashing it with SHA-256.
    ///
    /// This is useful for signing large messages or when you want
    /// a consistent message size regardless of input.
    pub fn sign_prehashed(&self, message: &[u8]) -> Signature {
        let hash = Sha256::digest(message);
        self.sign(&hash)
    }
}

impl Clone for SigningKey {
    fn clone(&self) -> Self {
        SigningKey(DalekSigningKey::from_bytes(&self.0.to_bytes()))
    }
}

impl std::fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SigningKey")
            .field("public_key", &self.verifying_key().to_hex())
            .finish_non_exhaustive()
    }
}

/// An Ed25519 verifying key (public key).
///
/// This key is used to verify signatures. It can be shared publicly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerifyingKey(DalekVerifyingKey);

impl VerifyingKey {
    /// The length of a verifying key in bytes.
    pub const LENGTH: usize = PUBLIC_KEY_LENGTH;

    /// Creates a verifying key from raw bytes.
    pub fn from_bytes(bytes: &[u8; Self::LENGTH]) -> Result<Self, SignatureError> {
        DalekVerifyingKey::from_bytes(bytes)
            .map(VerifyingKey)
            .map_err(|e| SignatureError::InvalidKey(e.to_string()))
    }

    /// Creates a verifying key from a byte slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, SignatureError> {
        if bytes.len() != Self::LENGTH {
            return Err(SignatureError::InvalidKey(format!(
                "expected {} bytes, got {}",
                Self::LENGTH,
                bytes.len()
            )));
        }
        let mut arr = [0u8; Self::LENGTH];
        arr.copy_from_slice(bytes);
        Self::from_bytes(&arr)
    }

    /// Creates a verifying key from a hex-encoded string.
    pub fn from_hex(hex: &str) -> Result<Self, SignatureError> {
        let bytes = hex::decode(hex).map_err(|e| SignatureError::HexError(e.to_string()))?;
        Self::from_slice(&bytes)
    }

    /// Returns the verifying key as raw bytes.
    pub fn to_bytes(&self) -> [u8; Self::LENGTH] {
        self.0.to_bytes()
    }

    /// Returns the verifying key as a hex-encoded string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    /// Verifies a signature against a message.
    ///
    /// Returns `Ok(())` if the signature is valid, or an error if verification fails.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), SignatureError> {
        self.0
            .verify(message, &signature.0)
            .map_err(|_| SignatureError::VerificationFailed)
    }

    /// Verifies a signature against a prehashed message.
    ///
    /// The message will be hashed with SHA-256 before verification.
    /// Use this when the message was signed with `sign_prehashed`.
    pub fn verify_prehashed(
        &self,
        message: &[u8],
        signature: &Signature,
    ) -> Result<(), SignatureError> {
        let hash = Sha256::digest(message);
        self.verify(&hash, signature)
    }
}

impl Serialize for VerifyingKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for VerifyingKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        VerifyingKey::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

/// A keypair containing both signing and verifying keys.
#[derive(Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl KeyPair {
    /// Generates a new random keypair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate();
        let verifying_key = signing_key.verifying_key();
        KeyPair {
            signing_key,
            verifying_key,
        }
    }

    /// Creates a keypair from an existing signing key.
    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();
        KeyPair {
            signing_key,
            verifying_key,
        }
    }

    /// Returns a reference to the signing key.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Returns the verifying key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }

    /// Signs a message using the signing key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Signs a message after hashing with SHA-256.
    pub fn sign_prehashed(&self, message: &[u8]) -> Signature {
        self.signing_key.sign_prehashed(message)
    }

    /// Verifies a signature using the verifying key.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), SignatureError> {
        self.verifying_key.verify(message, signature)
    }
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public_key", &self.verifying_key.to_hex())
            .finish_non_exhaustive()
    }
}

/// Signs data and returns the signature along with the verifying key.
///
/// This is a convenience function for one-shot signing operations
/// where you need both the signature and the public key.
pub fn sign_with_new_key(message: &[u8]) -> (Signature, VerifyingKey) {
    let keypair = KeyPair::generate();
    let signature = keypair.sign(message);
    (signature, keypair.verifying_key())
}

/// Verifies a signature against a message and verifying key.
///
/// This is a convenience function for one-shot verification operations.
pub fn verify(
    message: &[u8],
    signature: &Signature,
    verifying_key: &VerifyingKey,
) -> Result<(), SignatureError> {
    verifying_key.verify(message, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        let message = b"test message";
        let signature = keypair.sign(message);
        assert!(keypair.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_signing_key_generation() {
        let signing_key = SigningKey::generate();
        let verifying_key = signing_key.verifying_key();
        let message = b"hello world";
        let signature = signing_key.sign(message);
        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_verification_fails_for_wrong_message() {
        let keypair = KeyPair::generate();
        let signature = keypair.sign(b"original message");
        assert!(keypair.verify(b"different message", &signature).is_err());
    }

    #[test]
    fn test_verification_fails_for_wrong_key() {
        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::generate();
        let message = b"test";
        let signature = keypair1.sign(message);
        assert!(keypair2.verify(message, &signature).is_err());
    }

    #[test]
    fn test_signing_key_from_bytes() {
        let original = SigningKey::generate();
        let bytes = original.to_bytes();
        let restored = SigningKey::from_bytes(&bytes).unwrap();

        let message = b"test";
        let sig1 = original.sign(message);
        let sig2 = restored.sign(message);

        // Ed25519 signatures are deterministic
        assert_eq!(sig1.to_bytes(), sig2.to_bytes());
    }

    #[test]
    fn test_verifying_key_from_bytes() {
        let signing_key = SigningKey::generate();
        let verifying_key = signing_key.verifying_key();
        let bytes = verifying_key.to_bytes();
        let restored = VerifyingKey::from_bytes(&bytes).unwrap();

        let message = b"test";
        let signature = signing_key.sign(message);

        assert!(restored.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_hex_encoding() {
        let signing_key = SigningKey::generate();
        let hex = signing_key.to_hex();
        let restored = SigningKey::from_hex(&hex).unwrap();

        assert_eq!(signing_key.to_bytes(), restored.to_bytes());
    }

    #[test]
    fn test_signature_hex() {
        let keypair = KeyPair::generate();
        let signature = keypair.sign(b"test");
        let hex = signature.to_hex();
        let restored = Signature::from_hex(&hex).unwrap();

        assert_eq!(signature.to_bytes(), restored.to_bytes());
    }

    #[test]
    fn test_prehashed_signing() {
        let keypair = KeyPair::generate();
        let message = b"a very long message that would benefit from prehashing";
        let signature = keypair.sign_prehashed(message);
        assert!(keypair
            .verifying_key()
            .verify_prehashed(message, &signature)
            .is_ok());
    }

    #[test]
    fn test_sign_with_new_key() {
        let message = b"test message";
        let (signature, verifying_key) = sign_with_new_key(message);
        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_verifying_key_serialization() {
        let keypair = KeyPair::generate();
        let verifying_key = keypair.verifying_key();

        let json = serde_json::to_string(&verifying_key).unwrap();
        let restored: VerifyingKey = serde_json::from_str(&json).unwrap();

        assert_eq!(verifying_key, restored);
    }

    #[test]
    fn test_signature_serialization() {
        let keypair = KeyPair::generate();
        let signature = keypair.sign(b"test");

        let json = serde_json::to_string(&signature).unwrap();
        let restored: Signature = serde_json::from_str(&json).unwrap();

        assert_eq!(signature, restored);
    }

    #[test]
    fn test_invalid_signature_length() {
        let result = Signature::from_slice(&[0u8; 32]);
        assert!(matches!(result, Err(SignatureError::InvalidSignature(_))));
    }

    #[test]
    fn test_invalid_key_length() {
        let result = VerifyingKey::from_slice(&[0u8; 16]);
        assert!(matches!(result, Err(SignatureError::InvalidKey(_))));
    }

    #[test]
    fn test_deterministic_signatures() {
        let signing_key = SigningKey::generate();
        let message = b"test message";

        let sig1 = signing_key.sign(message);
        let sig2 = signing_key.sign(message);

        // Ed25519-dalek uses deterministic nonce generation
        assert_eq!(sig1.to_bytes(), sig2.to_bytes());
    }

    #[test]
    fn test_empty_message() {
        let keypair = KeyPair::generate();
        let signature = keypair.sign(b"");
        assert!(keypair.verify(b"", &signature).is_ok());
    }

    #[test]
    fn test_large_message() {
        let keypair = KeyPair::generate();
        let message = vec![0xAB; 1024 * 1024]; // 1 MB
        let signature = keypair.sign(&message);
        assert!(keypair.verify(&message, &signature).is_ok());
    }
}
