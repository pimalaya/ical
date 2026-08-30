//! # REFRESH-INTERVAL
//!
//! The `REFRESH-INTERVAL` property: how often a client should refetch a
//! published calendar (RFC 7986 5.7).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `REFRESH-INTERVAL` property marker.
#[allow(non_camel_case_types)]
pub struct REFRESH_INTERVAL;

impl IcalPropSpec for REFRESH_INTERVAL {
    const KIND: IcalPropKind = IcalPropKind::RefreshInterval;

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
