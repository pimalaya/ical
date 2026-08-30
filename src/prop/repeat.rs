//! # REPEAT
//!
//! The `REPEAT` property: how many times an alarm repeats after it first fires
//! (RFC 5545 3.8.6.2).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `REPEAT` property marker.
pub struct REPEAT;

impl IcalPropSpec for REPEAT {
    const KIND: IcalPropKind = IcalPropKind::Repeat;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Integer]
    }
}
