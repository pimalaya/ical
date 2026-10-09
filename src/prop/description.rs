//! # DESCRIPTION
//!
//! The `DESCRIPTION` property: the long description of the component (RFC 5545
//! 3.8.1.5).
//!
//! Repeatable: a `VJOURNAL` may carry several (RFC 5545 3.6.3), and so may a
//! `VCALENDAR`, one per language (RFC 7986 5.2).

use crate::prop::{IcalPropKind, spec::IcalPropSpec};

/// The `DESCRIPTION` property marker.
pub struct DESCRIPTION;

impl IcalPropSpec for DESCRIPTION {
    const KIND: IcalPropKind = IcalPropKind::Description;
}
