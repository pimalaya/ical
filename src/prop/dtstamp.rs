//! # DTSTAMP
//!
//! The `DTSTAMP` property: when this instance of the component was written (RFC
//! 5545 3.8.7.2).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `DTSTAMP` property marker.
pub struct DTSTAMP;

impl IcalPropSpec for DTSTAMP {
    const KIND: IcalPropKind = IcalPropKind::DtStamp;

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
