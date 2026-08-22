use std::fmt::{Display, Formatter};
use std::io::IsTerminal;
use std::str::FromStr;

/// Log encoding. [`LogFormat::Auto`] is JSON in release builds or when stderr is not a TTY.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    #[default]
    Auto,
    Pretty,
    Json,
}

impl LogFormat {
    /// Whether events should be emitted as JSON.
    #[must_use]
    pub fn json_on_stderr(self) -> bool {
        match self {
            Self::Json => true,
            Self::Pretty => false,
            Self::Auto => cfg!(not(debug_assertions)) || !std::io::stderr().is_terminal(),
        }
    }
}

impl Display for LogFormat {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Auto => "auto",
            Self::Pretty => "pretty",
            Self::Json => "json",
        })
    }
}

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            _ => Err(format!("expected auto, pretty, or json, got {value}")),
        }
    }
}
