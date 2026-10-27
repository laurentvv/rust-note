//! Standard plugins: MIME Tools, Converter and Exporter.
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
    let val: i64 = text
        .trim()
        .parse()
        .map_err(|e| format!("Invalid integer: {e}"))?;
    Ok(format!("0x{val:X}"))
}

/// Converts hexadecimal text (with or without 0x prefix) to decimal integer (from NppConverter.dll).
pub fn hex_to_dec(text: &str) -> Result<String, String> {
    let clean = text
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    let val = i64::from_str_radix(clean, 16).map_err(|e| format!("Invalid hex integer: {e}"))?;
    Ok(val.to_string())
}

/// Quoted-printable encodes ASCII text (from mimeTools.dll).
#[must_use]
pub fn quoted_printable_encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if ((33..=126).contains(&byte) && byte != b'=')
            || matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
        {
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

/// Exports document content to stand-alone RTF document (from NppExport.dll).
#[must_use]
pub fn export_to_rtf(title: &str, content: &str) -> String {
    let mut out = String::from(
        "{\\rtf1\\ansi\\ansicpg1252\\deff0\\deflang1033{\\fonttbl{\\f0\\fnil\\fcharset0 Consolas;}}\n",
    );
    out.push_str("{\\colortbl ;\\red220\\green220\\blue220;\\red32\\green32\\blue32;\\red120\\green120\\blue120;}\n");
    out.push_str("\\viewkind4\\uc1\\pard\\f0\\fs20 ");

    // Header title
    out.push_str(&format!("\\b {} \\b0\\par\\par\n", rtf_escape(title)));

    for line in content.lines() {
        out.push_str(&rtf_escape(line));
        out.push_str("\\par\n");
    }
    out.push('}');
    out
}

fn rtf_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '\n' => out.push_str("\\par\n"),
            '\r' => {}
            _ if c.is_ascii() => out.push(c),
            _ => out.push_str(&format!("\\u{}?", c as u32)),
        }
    }
    out
}

/// Real-time multi-base converter tool (recovered from NppConverter.dll "Conversion Panel").
#[derive(Debug, Clone)]
pub struct ConversionPanelState {
    pub is_open: bool,
    pub dec_val: String,
    pub hex_val: String,
    pub bin_val: String,
    pub oct_val: String,
    pub char_val: String,
}

impl Default for ConversionPanelState {
    fn default() -> Self {
        let mut s = Self {
            is_open: false,
            dec_val: "65".to_string(),
            hex_val: "41".to_string(),
            bin_val: "01000001".to_string(),
            oct_val: "101".to_string(),
            char_val: "A".to_string(),
        };
        s.sync_from_dec("65");
        s
    }
}

impl ConversionPanelState {
    pub fn sync_from_dec(&mut self, text: &str) {
        self.dec_val = text.to_string();
        if let Ok(num) = text.trim().parse::<i64>() {
            self.hex_val = format!("{num:X}");
            self.bin_val = format!("{num:b}");
            self.oct_val = format!("{num:o}");
            if (32..=126).contains(&num) {
                self.char_val = (num as u8 as char).to_string();
            } else {
                self.char_val = "\\0".to_string();
            }
        }
    }

    pub fn sync_from_hex(&mut self, text: &str) {
        self.hex_val = text.to_string();
        let clean = text
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches("0X");
        if let Ok(num) = i64::from_str_radix(clean, 16) {
            self.dec_val = num.to_string();
            self.bin_val = format!("{num:b}");
            self.oct_val = format!("{num:o}");
            if (32..=126).contains(&num) {
                self.char_val = (num as u8 as char).to_string();
            } else {
                self.char_val = "\\0".to_string();
            }
        }
    }

    pub fn sync_from_bin(&mut self, text: &str) {
        self.bin_val = text.to_string();
        let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        if let Ok(num) = i64::from_str_radix(&clean, 2) {
            self.dec_val = num.to_string();
            self.hex_val = format!("{num:X}");
            self.oct_val = format!("{num:o}");
            if (32..=126).contains(&num) {
                self.char_val = (num as u8 as char).to_string();
            } else {
                self.char_val = "\\0".to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let orig = "rust-note 2026!";
        let encoded = base64_encode(orig);
        let decoded = base64_decode(&encoded).expect("decode failed");
        assert_eq!(orig, decoded);
    }

    #[test]
    fn test_url_roundtrip() {
        let orig = "https://rust-note.org/download/v0.1.html?q=rust port";
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

    #[test]
    fn test_export_to_rtf() {
        let rtf = export_to_rtf("doc.txt", "Hello World\nLine 2");
        assert!(rtf.starts_with("{\\rtf1"));
        assert!(rtf.contains("Hello World"));
        assert!(rtf.ends_with('}'));
    }

    #[test]
    fn test_conversion_panel_sync() {
        let mut panel = ConversionPanelState::default();
        panel.sync_from_dec("42");
        assert_eq!(panel.hex_val, "2A");
        assert_eq!(panel.bin_val, "101010");
        assert_eq!(panel.oct_val, "52");
        assert_eq!(panel.char_val, "*");

        panel.sync_from_hex("0xFF");
        assert_eq!(panel.dec_val, "255");
        assert_eq!(panel.bin_val, "11111111");

        panel.sync_from_bin("1000");
        assert_eq!(panel.dec_val, "8");
        assert_eq!(panel.hex_val, "8");
    }
}
