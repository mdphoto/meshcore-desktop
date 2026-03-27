//! HMAC-SHA256 pour l'authentification des messages et la dérivation de clés

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Calcule un HMAC-SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC accepte des clés de toute taille");
    mac.update(data);
    let result = mac.finalize();
    result.into_bytes().into()
}

/// Vérifie un HMAC-SHA256
pub fn hmac_sha256_verify(key: &[u8], data: &[u8], expected: &[u8; 32]) -> bool {
    let computed = hmac_sha256(key, data);
    // Comparaison en temps constant
    constant_time_eq(&computed, expected)
}

/// Dérive un PSK de canal hashtag à partir du nom (SHA-256, premiers 16 octets)
pub fn derive_hashtag_psk(name: &str) -> [u8; 16] {
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(name.as_bytes());
    let mut psk = [0u8; 16];
    psk.copy_from_slice(&hash[..16]);
    psk
}

/// Dérive un PSK communautaire par HMAC-SHA256
pub fn derive_community_psk(secret: &[u8], name: &[u8]) -> [u8; 16] {
    let hmac = hmac_sha256(secret, name);
    let mut psk = [0u8; 16];
    psk.copy_from_slice(&hmac[..16]);
    psk
}

fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sha256_rfc4231_vector1() {
        // RFC 4231 Test Case 1
        let key = [0x0b; 20];
        let data = b"Hi There";
        let expected = hex::decode(
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        ).unwrap();

        let result = hmac_sha256(&key, data);
        assert_eq!(result, expected.as_slice());
    }

    #[test]
    fn test_verify_valid() {
        let key = b"secret key";
        let data = b"message";
        let mac = hmac_sha256(key, data);
        assert!(hmac_sha256_verify(key, data, &mac));
    }

    #[test]
    fn test_verify_invalid() {
        let key = b"secret key";
        let data = b"message";
        let mut mac = hmac_sha256(key, data);
        mac[0] ^= 0xff;
        assert!(!hmac_sha256_verify(key, data, &mac));
    }

    #[test]
    fn test_derive_hashtag_psk() {
        let psk1 = derive_hashtag_psk("test-channel");
        let psk2 = derive_hashtag_psk("test-channel");
        let psk3 = derive_hashtag_psk("other-channel");
        assert_eq!(psk1, psk2);
        assert_ne!(psk1, psk3);
        assert_eq!(psk1.len(), 16);
    }
}
