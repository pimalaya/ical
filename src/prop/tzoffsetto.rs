//! # TZOFFSETTO
//!
//! The `TZOFFSETTO` property: the UTC offset in force after it takes effect
//! (RFC 5545 3.8.3.4).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `TZOFFSETTO` property marker.
pub struct TZOFFSETTO;

impl IcalPropSpec for TZOFFSETTO {
    const KIND: IcalPropKind = IcalPropKind::TzOffsetTo;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::UtcOffset]
    }
}
