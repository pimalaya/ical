//! # Geographic-position value
//!
//! The decoded geographic-position value kind.
//!
//! Backs the `GEO` property (RFC 5545 3.8.1.6): a latitude and a longitude,
//! each a FLOAT (RFC 5545 3.3.7), separated on the wire by a semicolon. Both
//! components are kept as their raw text; the crate does not parse them into
//! numbers.

use alloc::borrow::Cow;

/// A decoded geographic position (latitude and longitude), each kept as its raw
/// text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalGeo<'a> {
    /// The latitude component (a FLOAT), kept as its raw text.
    pub latitude: Cow<'a, str>,
    /// The longitude component (a FLOAT), kept as its raw text.
    pub longitude: Cow<'a, str>,
}
