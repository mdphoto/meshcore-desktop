//! Chiffrement AES-128 en mode ECB pour les canaux MeshCore

use aes::Aes128;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};

/// Chiffre des données avec AES-128-ECB
///
/// Les données sont paddées avec des zéros pour atteindre un multiple de 16 octets.
pub fn aes128_ecb_encrypt(key: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let padded_len = plaintext.len().div_ceil(16) * 16;
    let mut data = vec![0u8; padded_len];
    data[..plaintext.len()].copy_from_slice(plaintext);

    for chunk in data.chunks_exact_mut(16) {
        let block = GenericArray::from_mut_slice(chunk);
        cipher.encrypt_block(block);
    }
    data
}

/// Déchiffre des données avec AES-128-ECB
pub fn aes128_ecb_decrypt(key: &[u8; 16], ciphertext: &[u8]) -> Result<Vec<u8>, AesError> {
    if !ciphertext.len().is_multiple_of(16) {
        return Err(AesError::InvalidLength(ciphertext.len()));
    }

    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut data = ciphertext.to_vec();

    for chunk in data.chunks_exact_mut(16) {
        let block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(block);
    }
    Ok(data)
}

#[derive(Debug, thiserror::Error)]
pub enum AesError {
    #[error("Longueur de ciphertext invalide : {0} (doit être multiple de 16)")]
    InvalidLength(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let plaintext = b"Hello MeshCore!!"; // exactement 16 octets

        let encrypted = aes128_ecb_encrypt(&key, plaintext);
        assert_eq!(encrypted.len(), 16);

        let decrypted = aes128_ecb_decrypt(&key, &encrypted).unwrap();
        assert_eq!(&decrypted[..plaintext.len()], plaintext);
    }

    #[test]
    fn test_padding() {
        let key = [0u8; 16];
        let plaintext = b"short";

        let encrypted = aes128_ecb_encrypt(&key, plaintext);
        assert_eq!(encrypted.len(), 16); // paddé à 16

        let decrypted = aes128_ecb_decrypt(&key, &encrypted).unwrap();
        assert_eq!(&decrypted[..plaintext.len()], plaintext);
    }
}
