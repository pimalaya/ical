//! # Escaping mode
//!
//! The one place the codec consults the calendar version.
//!
//! Value escaping differs between vCalendar 1.0 (versit) and iCalendar 2.0
//! (RFC 5545 3.3.11), and parameter encoding (RFC 6868) exists only from 2.0.
//!
//! A value node and a parameter node therefore each carry an [`Escaper`]
//! telling the sibling [`escape`](crate::tree::codec::escape) and
//! [`unescape`](crate::tree::codec::unescape) codecs which rules to apply.

use crate::version::IcalVersion;

/// The escaping rules to apply, selected by the calendar version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Escaper {
    /// vCalendar 1.0 (versit): `\;` is resolved on read and a backslash before
    /// anything else is literal; writing also escapes a newline, which versit
    /// has none of its own for and which raw would end the line.
    V1_0,
    /// iCalendar 2.0 (RFC 5545 3.3.11): the value escapes `\\`, `\,`, `\;` and
    /// `\n`, plus the RFC 6868 parameter value encoding.
    #[default]
    Modern,
}

impl Escaper {
    /// The escaping rules a calendar of `version` uses.
    pub fn for_version(version: IcalVersion) -> Self {
        match version {
            IcalVersion::V1_0 => Self::V1_0,
            IcalVersion::V2_0 => Self::Modern,
        }
    }

    /// The escaping rules for a raw `VERSION` wire string (e.g. `"1.0"`).
    pub fn for_version_str(version: &str) -> Self {
        match version.parse() {
            Ok(IcalVersion::V1_0) => Self::V1_0,
            _ => Self::Modern,
        }
    }

    /// Whether this version carries the RFC 6868 parameter value encoding,
    /// which updates RFC 5545 and so reaches iCalendar 2.0 alone: vCalendar 1.0
    /// predates it, and a caret in one of its parameters is a literal caret.
    pub fn has_param_encoding(self) -> bool {
        matches!(self, Self::Modern)
    }
}
