//! Gestion des identités Ed25519 pour la signature des messages

use ed25519_dalek::{
    Signature, Signer, SigningKey, Verifier, VerifyingKey,
};
use rand::rngs::OsRng;

/// Paire de clés Ed25519
#[derive(Debug)]
pub struct Ed25519Keypair {
    signing_key: SigningKey,
}

impl Ed25519Keypair {
    /// Génère une nouvelle paire de clés aléatoire
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Restaure une paire de clés à partir de la clé privée (32 octets)
    pub fn from_secret(secret: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(secret);
        Self { signing_key }
    }

    /// Retourne la clé publique (32 octets)
    pub fn public_key(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Retourne la clé privée (32 octets)
    pub fn secret_key(&self) -> &[u8; 32] {
        self.signing_key.as_bytes()
    }

    /// Signe un message
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature = self.signing_key.sign(message);
        signature.to_bytes()
    }
}

/// Signe un message avec une clé privée brute
pub fn sign(secret_key: &[u8; 32], message: &[u8]) -> [u8; 64] {
    let signing_key = SigningKey::from_bytes(secret_key);
    let signature = signing_key.sign(message);
    signature.to_bytes()
}

/// Vérifie une signature Ed25519
pub fn verify(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(public_key) else {
        return false;
    };
    let sig = Signature::from_bytes(signature);
    verifying_key.verify(message, &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_sign_verify() {
        let keypair = Ed25519Keypair::generate();
        let message = b"MeshCore test message";

        let signature = keypair.sign(message);
        assert!(verify(&keypair.public_key(), message, &signature));
    }

    #[test]
    fn test_wrong_message_fails() {
        let keypair = Ed25519Keypair::generate();
        let signature = keypair.sign(b"correct message");
        assert!(!verify(&keypair.public_key(), b"wrong message", &signature));
    }

    #[test]
    fn test_restore_from_secret() {
        let keypair1 = Ed25519Keypair::generate();
        let keypair2 = Ed25519Keypair::from_secret(keypair1.secret_key());

        assert_eq!(keypair1.public_key(), keypair2.public_key());

        let message = b"test";
        let sig = keypair1.sign(message);
        assert!(verify(&keypair2.public_key(), message, &sig));
    }
}
