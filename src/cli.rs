//! Notepad++ command-line arguments parser.

use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct CliArgs {
    pub multi_inst: bool,
    pub no_plugin: bool,
    pub no_session: bool,
    pub no_tab_bar: bool,
    pub read_only: bool,
    pub always_on_top: bool,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub pos: Option<usize>,
    pub language: Option<String>,
    pub files: Vec<PathBuf>,
}

impl CliArgs {
    /// Parses CLI arguments adhering to Notepad++ single-dash syntax.
    #[must_use]
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let mut cli = Self::default();
        let raw: Vec<String> = args.into_iter().map(Into::into).collect();

        for arg in raw {
            if arg == "-multiInst" || arg == "--multiInst" {
                cli.multi_inst = true;
            } else if arg == "-noPlugin" || arg == "--noPlugin" {
                cli.no_plugin = true;
            } else if arg == "-nosession" || arg == "--nosession" {
                cli.no_session = true;
            } else if arg == "-notabbar" || arg == "--notabbar" {
                cli.no_tab_bar = true;
            } else if arg == "-ro" || arg == "--ro" {
                cli.read_only = true;
            } else if arg == "-alwaysOnTop" || arg == "--alwaysOnTop" {
                cli.always_on_top = true;
            } else if let Some(val) = arg.strip_prefix("-n=").or_else(|| arg.strip_prefix("-n")) {
                if let Ok(num) = val.parse::<usize>() {
                    cli.line = Some(num);
                }
            } else if let Some(val) = arg.strip_prefix("-c=").or_else(|| arg.strip_prefix("-c")) {
                if let Ok(num) = val.parse::<usize>() {
                    cli.col = Some(num);
                }
            } else if let Some(val) = arg.strip_prefix("-p=").or_else(|| arg.strip_prefix("-p")) {
                if let Ok(num) = val.parse::<usize>() {
                    cli.pos = Some(num);
                }
            } else if let Some(val) = arg.strip_prefix("-l=").or_else(|| arg.strip_prefix("-l")) {
                cli.language = Some(val.to_string());
            } else if !arg.starts_with('-') {
                cli.files.push(PathBuf::from(arg));
            }
        }

        cli
    }

    #[must_use]
    pub fn parse() -> Self {
        Self::parse_from(std::env::args().skip(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notepadpp_cli_flags() {
        let args = vec![
            "-nosession",
            "-multiInst",
            "-ro",
            "-n42",
            "-c10",
            "-lrust",
            "test.txt",
        ];
        let parsed = CliArgs::parse_from(args);
        assert!(parsed.no_session);
        assert!(parsed.multi_inst);
        assert!(parsed.read_only);
        assert_eq!(parsed.line, Some(42));
        assert_eq!(parsed.col, Some(10));
        assert_eq!(parsed.language.as_deref(), Some("rust"));
        assert_eq!(parsed.files, vec![PathBuf::from("test.txt")]);
    }
}
