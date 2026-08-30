//! # Property cardinality
//!
//! How many times a property may appear in its component, per RFC 5545 section
//! 3.6.

/// The RFC 5545 property multiplicity: how many times a property may appear in
/// its component. Property multiplicity, not value structure, so it is not
/// derivable from the value kind (`SUMMARY` and `COMMENT` are both text but
/// differ).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalPropCardinality {
    /// Exactly one (required, single).
    ExactlyOne,
    /// At most one (optional, single).
    AtMostOne,
    /// One or more (required, repeatable).
    OneOrMore,
    /// Any number, including zero (optional, repeatable).
    Any,
}
