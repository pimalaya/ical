//! # Request-status value
//!
//! The decoded request-status value kind.
//!
//! Backs the `REQUEST-STATUS` property (RFC 5545 3.8.8.3): a status code, a
//! human-readable description, and optional extra data, the three components
//! separated on the wire by semicolons. Each component is kept as its raw text;
//! the optional extra data is an empty [`Cow`] when absent. Pure data, no
//! escaping; the owning property's wire name lives on
//! [`crate::prop::IcalProp::name`].

use alloc::borrow::Cow;

/// A decoded request-status value (code, description, and optional extra data),
/// each kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalRequestStatus<'a> {
    /// The hierarchical status code (e.g. `2.0`), kept as its raw text.
    pub code: Cow<'a, str>,
    /// The human-readable description of the status.
    pub description: Cow<'a, str>,
    /// Optional extra data; an empty [`Cow`] when absent.
    pub extra: Cow<'a, str>,
}
