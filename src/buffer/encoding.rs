//! Character encodings support: UTF-8, UTF-8-BOM, UTF-16LE, UTF-16BE, Windows-1252 (ANSI).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Encoding {
    #[default]
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    Ansi, // Windows-1252
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8 => write!(f, "UTF-8"),
            Self::Utf8Bom => write!(f, "UTF-8-BOM"),
            Self::Utf16Le => write!(f, "UTF-16 LE"),
            Self::Utf16Be => write!(f, "UTF-16 BE"),
            Self::Ansi => write!(f, "ANSI (Windows-1252)"),
        }
    }
}

/// Decodes binary file data into a Rust String and detected Encoding.
pub fn decode(bytes: &[u8]) -> (String, Encoding) {
    if bytes.is_empty() {
        return (String::new(), Encoding::Utf8);
    }

    // Check BOMs
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        let (cow, _, _) = encoding_rs::UTF_8.decode(&bytes[3..]);
        return (cow.into_owned(), Encoding::Utf8Bom);
    }

    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let (cow, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return (cow.into_owned(), Encoding::Utf16Le);
    }

    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let (cow, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return (cow.into_owned(), Encoding::Utf16Be);
    }

    // Attempt standard UTF-8
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_owned(), Encoding::Utf8),
        Err(_) => {
            // Fallback to Windows-1252 (ANSI)
            let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            (cow.into_owned(), Encoding::Ansi)
        }
    }
}

/// Encodes text to binary with the selected Encoding.
pub fn encode(text: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        Encoding::Utf8 => text.as_bytes().to_vec(),
        Encoding::Utf8Bom => {
            let mut out = vec![0xEF, 0xBB, 0xBF];
            out.extend_from_slice(text.as_bytes());
            out
        }
        Encoding::Utf16Le => {
            let mut out = vec![0xFF, 0xFE];
            for c in text.encode_utf16() {
                out.extend_from_slice(&c.to_le_bytes());
            }
            out
        }
        Encoding::Utf16Be => {
            let mut out = vec![0xFE, 0xFF];
            for c in text.encode_utf16() {
                out.extend_from_slice(&c.to_be_bytes());
            }
            out
        }
        Encoding::Ansi => {
            let (cow, _, _) = encoding_rs::WINDOWS_1252.encode(text);
            cow.into_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_roundtrip() {
        let original = "Hello World! Été, café, emoji: 🦀";
        let bytes = encode(original, Encoding::Utf8);
        let (decoded, enc) = decode(&bytes);
        assert_eq!(enc, Encoding::Utf8);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_utf8_bom_roundtrip() {
        let original = "BOM test text";
        let bytes = encode(original, Encoding::Utf8Bom);
        let (decoded, enc) = decode(&bytes);
        assert_eq!(enc, Encoding::Utf8Bom);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_utf16le_roundtrip() {
        let original = "UTF-16 LE test";
        let bytes = encode(original, Encoding::Utf16Le);
        let (decoded, enc) = decode(&bytes);
        assert_eq!(enc, Encoding::Utf16Le);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_ansi_roundtrip() {
        let original = "Ansi Windows-1252: accents é à ç";
        let bytes = encode(original, Encoding::Ansi);
        let (decoded, enc) = decode(&bytes);
        assert_eq!(enc, Encoding::Ansi);
        assert_eq!(decoded, original);
    }
}
