//! Standard Notepad++ plugins: MIME Tools, Converter and Exporter.
//! Recovered from `mimeTools.dll`, `NppConverter.dll` and `NppExport.dll`.

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

/// Converts text to hex space-separated representation (from NppConverter.dll).
#[must_use]
pub fn ascii_to_hex(text: &str) -> String {
    text.bytes()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Decodes space-separated or continuous hex string into ASCII (from NppConverter.dll).
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

/// Converts decimal integer text to hexadecimal format (0x...) (from NppConverter.dll).
pub fn dec_to_hex(text: &str) -> Result<String, String> {
    let val: i64 = text.trim().parse().map_err(|e| format!("Invalid integer: {e}"))?;
    Ok(format!("0x{val:X}"))
}

/// Converts hexadecimal text (with or without 0x prefix) to decimal integer (from NppConverter.dll).
pub fn hex_to_dec(text: &str) -> Result<String, String> {
    let clean = text.trim().trim_start_matches("0x").trim_start_matches("0X");
    let val = i64::from_str_radix(clean, 16).map_err(|e| format!("Invalid hex integer: {e}"))?;
    Ok(val.to_string())
}

/// Quoted-printable encodes ASCII text (from mimeTools.dll).
#[must_use]
pub fn quoted_printable_encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if ((33..=126).contains(&byte) && byte != b'=') || matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("={byte:02X}"));
        }
    }
    out
}

/// Exports document content to stand-alone syntax-styled HTML (from NppExport.dll).
#[must_use]
pub fn export_to_html(title: &str, content: &str, lang_name: &str) -> String {
    let escaped_title = html_escape(title);
    let mut rows = String::new();

    for (zero_idx, line) in content.lines().enumerate() {
        let line_num = zero_idx + 1;
        let escaped_line = html_escape(line);
        rows.push_str(&format!(
            "<tr><td class=\"ln\">{line_num}</td><td class=\"code\">{escaped_line}</td></tr>\n"
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{escaped_title}</title>
<style>
body {{ font-family: 'Consolas', 'Courier New', monospace; font-size: 13px; background: #202020; color: #dcdccc; padding: 16px; margin: 0; }}
h2 {{ color: #dfc47d; border-bottom: 1px solid #444; padding-bottom: 8px; }}
.badge {{ background: #2a2a2a; color: #7f9f7f; padding: 2px 8px; border-radius: 4px; font-size: 11px; }}
table {{ border-collapse: collapse; width: 100%; }}
td.ln {{ width: 40px; text-align: right; padding-right: 16px; color: #666; user-select: none; border-right: 1px solid #333; }}
td.code {{ padding-left: 16px; white-space: pre-wrap; }}
</style>
</head>
<body>
<h2>{escaped_title} <span class="badge">{lang_name}</span></h2>
<table>
{rows}</table>
</body>
</html>"#
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let orig = "Notepad++ Rust Port 2026!";
        let encoded = base64_encode(orig);
        let decoded = base64_decode(&encoded).expect("decode failed");
        assert_eq!(orig, decoded);
    }

    #[test]
    fn test_url_roundtrip() {
        let orig = "https://notepad-plus-plus.org/download/v8.7.html?q=rust port";
        let encoded = url_encode(orig);
        let decoded = url_decode(&encoded).expect("decode failed");
        assert_eq!(orig, decoded);
    }

    #[test]
    fn test_hex_roundtrip() {
        let orig = "Hello";
        let hex = ascii_to_hex(orig);
        assert_eq!(hex, "48 65 6C 6C 6F");
        let back = hex_to_ascii(&hex).expect("hex decode failed");
        assert_eq!(orig, back);
    }

    #[test]
    fn test_dec_hex_conversion() {
        assert_eq!(dec_to_hex("255").unwrap(), "0xFF");
        assert_eq!(hex_to_dec("0xFF").unwrap(), "255");
        assert_eq!(hex_to_dec("2A").unwrap(), "42");
    }

    #[test]
    fn test_quoted_printable() {
        let input = "Hello World!";
        let encoded = quoted_printable_encode(input);
        assert_eq!(encoded, "Hello World!");
    }

    #[test]
    fn test_export_to_html() {
        let html = export_to_html("test.rs", "fn main() {\n    println!(\"hi\");\n}", "Rust");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("test.rs"));
        assert!(html.contains("Rust"));
        assert!(html.contains("td class=\"ln\">1</td>"));
    }
}
