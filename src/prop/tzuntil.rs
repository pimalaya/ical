//! # TZUNTIL
//!
//! The `TZUNTIL` property: the UTC instant a `VTIMEZONE` definition stops
//! being valid at, exclusive (RFC 7808 7.1).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `TZUNTIL` property marker.
pub struct TZUNTIL;

impl IcalPropSpec for TZUNTIL {
    const KIND: IcalPropKind = IcalPropKind::TzUntil;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
