//! # TZID
//!
//! The `TZID` property: the identifier a `VTIMEZONE` answers to, and the zone a
//! date-time is read in (RFC 5545 3.8.3.1).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `TZID` property marker.
pub struct TZID;

impl IcalPropSpec for TZID {
    const KIND: IcalPropKind = IcalPropKind::TzId;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
