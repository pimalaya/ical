//! # Whole-calendar strict layer
//!
//! The "strict out" half of the crate, both facing the decoded calendar: the
//! [`builder`] constructs one property at a time against its spec, and
//! [`validate`] checks a whole [`Ical`](crate::ical::Ical) against the RFC 6350
//! rules and mints a [`IcalValid`](crate::valid::IcalValid) proof. Both share
//! the same per-property check, and both live here so the tree's read side
//! ([`cst`](crate::tree::cst), [`codec`](crate::tree::codec)) and its write
//! side stay visibly separate.

pub mod builder;
pub mod validate;
