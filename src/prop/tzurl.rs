//! # TZURL
//!
//! The `TZURL` property: where the up-to-date `VTIMEZONE` definition lives (RFC
//! 5545 3.8.3.5).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `TZURL` property marker.
pub struct TZURL;

impl IcalPropSpec for TZURL {
    const KIND: IcalPropKind = IcalPropKind::TzUrl;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
