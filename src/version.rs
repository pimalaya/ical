//! # Version
//!
//! The calendar version value and its name vocabulary.
//!
//! [`IcalVersion`] is the decoded `VERSION` line: one of the two defined
//! versions (vCalendar 1.0 / iCalendar 2.0). An unrecognised or missing version
//! is normalised to [`V2_0`](IcalVersion::V2_0) at decode time; preserving the
//! raw `VERSION` line byte for byte is the syntax tree's job, not the model's.
//! The version sits apart from the other properties because the syntax tree
//! treats it as a fixed part of the calendar envelope rather than a free
//! property. Pure model, no syntax dependency.

use core::{error, fmt, ops, str};

use alloc::string::{String, ToString};

/// Parse iCalendar version error.
#[derive(Debug)]
pub struct ParseIcalVersionError(
    /// The iCalendar version that cannot be parsed.
    String,
);

impl fmt::Display for ParseIcalVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse iCalendar version `{}`", self.0)
    }
}

impl error::Error for ParseIcalVersionError {}

/// The iCalendar version: one of the two defined versions. An unrecognised or
/// missing version normalises to [`V2_0`](Self::V2_0) (see the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalVersion {
    /// vCalendar 1.0 (versit/IMC).
    V1_0,
    /// iCalendar 2.0 (RFC 5545, and its extensions).
    V2_0,
}

impl str::FromStr for IcalVersion {
    type Err = ParseIcalVersionError;

    /// The defined version for a wire string (`1.0`, `2.0`).
    fn from_str(version: &str) -> Result<Self, Self::Err> {
        match version {
            "1.0" => Ok(Self::V1_0),
            "2.0" => Ok(Self::V2_0),
            _ => Err(ParseIcalVersionError(version.to_string())),
        }
    }
}

impl ops::Deref for IcalVersion {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::V1_0 => "1.0",
            Self::V2_0 => "2.0",
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::version::IcalVersion;

    #[test]
    fn maps_known_wire_strings_both_ways() {
        assert_eq!("1.0".parse().ok(), Some(IcalVersion::V1_0));
        assert_eq!(IcalVersion::V2_0.to_string(), "2.0");
        assert_eq!(&*IcalVersion::V2_0, "2.0");
    }

    #[test]
    fn rejects_unknown_versions() {
        let error = "5.0".parse::<IcalVersion>().unwrap_err();
        assert!(error.to_string().contains("5.0"));
    }
}
