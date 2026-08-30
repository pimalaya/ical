//! # TZOFFSETFROM
//!
//! The `TZOFFSETFROM` property: the UTC offset in force before an observance
//! takes effect (RFC 5545 3.8.3.3).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `TZOFFSETFROM` property marker.
pub struct TZOFFSETFROM;

impl IcalPropSpec for TZOFFSETFROM {
    const KIND: IcalPropKind = IcalPropKind::TzOffsetFrom;

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
