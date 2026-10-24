//! Standard Notepad++ plugins: MIME Tools and Converter.

use base64::prelude::*;

/// Base64 encodes input text.
#[must_use]
pub fn base64_encode(text: &str) -> String {
    BASE64_STANDARD.encode(text.as_bytes())
}

/// Base64 decodes input text.
pub fn base64_decode(text: &str) -> Result<String, String> {
    let clean = text.trim();
    let bytes = BASE64_STANDARD
        .decode(clean)
        .map_err(|e| format!("Base64 decoding failed: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("Decoded bytes are not valid UTF-8: {e}"))
}

/// URL encodes input text.
#[must_use]
pub fn url_encode(text: &str) -> String {
    urlencoding::encode(text).into_owned()
}

/// URL decodes input text.
pub fn url_decode(text: &str) -> Result<String, String> {
    urlencoding::decode(text)
        .map(|cow| cow.into_owned())
        .map_err(|e| format!("URL decoding failed: {e}"))
}

/// Converts text to hex space-separated representation.
#[must_use]
pub fn ascii_to_hex(text: &str) -> String {
    text.bytes()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Decodes space-separated or continuous hex string into ASCII.
pub fn hex_to_ascii(text: &str) -> Result<String, String> {
    let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if !clean.len().is_multiple_of(2) {
        return Err("Hex string length must be an even number of characters".to_string());
    }

    let mut bytes = Vec::with_capacity(clean.len() / 2);
    for i in (0..clean.len()).step_by(2) {
        let byte = u8::from_str_radix(&clean[i..i + 2], 16)
            .map_err(|e| format!("Invalid hex byte '{}': {e}", &clean[i..i + 2]))?;
        bytes.push(byte);
    }

    String::from_utf8(bytes).map_err(|e| format!("Hex bytes are not valid UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let src = "Notepad++ Clean-Room";
        let enc = base64_encode(src);
        let dec = base64_decode(&enc).unwrap();
        assert_eq!(dec, src);
    }

    #[test]
    fn test_url_roundtrip() {
        let src = "Hello World & Notepad++? Yes!";
        let enc = url_encode(src);
        let dec = url_decode(&enc).unwrap();
        assert_eq!(dec, src);
    }

    #[test]
    fn test_hex_roundtrip() {
        let src = "Rust";
        let hex = ascii_to_hex(src);
        assert_eq!(hex, "52 75 73 74");
        let dec = hex_to_ascii(&hex).unwrap();
        assert_eq!(dec, src);
    }
}
