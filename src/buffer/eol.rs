//! End of Line (EOL) formats: Windows (CRLF), Unix (LF), Macintosh (CR).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EolFormat {
    #[default]
    Windows, // \r\n
    Unix, // \n
    Mac,  // \r
}

impl fmt::Display for EolFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Windows => write!(f, "Windows (CR LF)"),
            Self::Unix => write!(f, "Unix (LF)"),
            Self::Mac => write!(f, "Macintosh (CR)"),
        }
    }
}

impl EolFormat {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "\r\n",
            Self::Unix => "\n",
            Self::Mac => "\r",
        }
    }
}

/// Detects the predominant EOL format in the text.
#[must_use]
pub fn detect(text: &str) -> EolFormat {
    let mut crlf_count = 0usize;
    let mut lf_count = 0usize;
    let mut cr_count = 0usize;

    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                crlf_count += 1;
                i += 2;
                continue;
            }
            cr_count += 1;
        } else if bytes[i] == b'\n' {
            lf_count += 1;
        }
        i += 1;
    }

    if crlf_count >= lf_count && crlf_count >= cr_count && crlf_count > 0 {
        EolFormat::Windows
    } else if lf_count >= cr_count && lf_count > 0 {
        EolFormat::Unix
    } else if cr_count > 0 {
        EolFormat::Mac
    } else {
        EolFormat::Windows // default
    }
}

/// Converts all EOL markers in the text to the target EOL format.
#[must_use]
pub fn convert(text: &str, target: EolFormat) -> String {
    let target_eol = target.as_str();
    let mut result = String::with_capacity(text.len());

    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                result.push_str(target_eol);
                i += 2;
                continue;
            }
            result.push_str(target_eol);
        } else if bytes[i] == b'\n' {
            result.push_str(target_eol);
        } else {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eol_detect_and_convert() {
        let text_crlf = "Line 1\r\nLine 2\r\nLine 3";
        assert_eq!(detect(text_crlf), EolFormat::Windows);

        let text_lf = convert(text_crlf, EolFormat::Unix);
        assert_eq!(detect(&text_lf), EolFormat::Unix);
        assert_eq!(text_lf, "Line 1\nLine 2\nLine 3");

        let text_mac = convert(&text_lf, EolFormat::Mac);
        assert_eq!(detect(&text_mac), EolFormat::Mac);
        assert_eq!(text_mac, "Line 1\rLine 2\rLine 3");
    }
}
