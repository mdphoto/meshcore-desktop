//! Couche cryptographique MeshCore
//!
//! Fournit AES-128-ECB, HMAC-SHA256 et Ed25519 pour le chiffrement
//! des canaux et la signature des messages.

pub mod aes_ecb;
pub mod hmac_sha256;
pub mod identity;

pub use aes_ecb::{aes128_ecb_decrypt, aes128_ecb_encrypt};
pub use hmac_sha256::{hmac_sha256, hmac_sha256_verify};
pub use identity::{Ed25519Keypair, sign, verify};
