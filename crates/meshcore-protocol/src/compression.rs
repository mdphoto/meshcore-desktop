//! Compression SMAZ pour les messages texte courts

/// Préfixe indiquant un message compressé SMAZ
pub const SMAZ_PREFIX: &str = "s:";

/// Compresse un texte avec SMAZ si le résultat est plus petit
pub fn compress(text: &str) -> Vec<u8> {
    let compressed = smaz::compress(text.as_bytes());
    if compressed.len() < text.len() {
        let mut result = SMAZ_PREFIX.as_bytes().to_vec();
        result.extend_from_slice(&compressed);
        result
    } else {
        text.as_bytes().to_vec()
    }
}

/// Décompresse un message (détecte automatiquement le préfixe SMAZ)
pub fn decompress(data: &[u8]) -> Result<String, DecompressError> {
    if data.starts_with(SMAZ_PREFIX.as_bytes()) {
        let decompressed = smaz::decompress(&data[SMAZ_PREFIX.len()..])
            .map_err(DecompressError::Smaz)?;
        String::from_utf8(decompressed).map_err(DecompressError::Utf8)
    } else {
        String::from_utf8(data.to_vec()).map_err(DecompressError::Utf8)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecompressError {
    #[error("Erreur de décompression SMAZ : {0}")]
    Smaz(smaz::DecompressError),
    #[error("Texte UTF-8 invalide : {0}")]
    Utf8(std::string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let text = "hello world this is a test message";
        let compressed = compress(text);
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, text);
    }

    #[test]
    fn test_short_text_not_compressed() {
        let text = "hi";
        let compressed = compress(text);
        // Court texte : pas de préfixe SMAZ car pas de gain
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, text);
    }
}
