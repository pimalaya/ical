//! # Escaping mode
//!
//! The one place the codec consults the calendar version: value escaping
//! differs between vCalendar 1.0 (versit) and iCalendar 2.0 (RFC 5545 3.3.11),
//! so a value node carries an [`Escaper`] telling the sibling
//! [`escape`](crate::tree::codec::escape) and
//! [`unescape`](crate::tree::codec::unescape) codecs which rules to apply.

use crate::version::IcalVersion;

/// The value-escaping rules to apply, selected by the calendar version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Escaper {
    /// vCalendar 1.0 (versit): `\;` is resolved on read and a backslash before
    /// anything else is literal; writing also escapes a newline, which versit
    /// has none of its own for and which raw would end the line.
    V1_0,
    /// iCalendar 2.0 (RFC 5545 3.3.11): `\\`, `\,`, `\;` and `\n`.
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
}
