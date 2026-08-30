//! # DURATION
//!
//! The `DURATION` property: how long the component lasts, an alternative to
//! `DTEND` (RFC 5545 3.8.2.5).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `DURATION` property marker.
pub struct DURATION;

impl IcalPropSpec for DURATION {
    const KIND: IcalPropKind = IcalPropKind::Duration;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Duration]
    }
}
