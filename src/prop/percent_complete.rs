//! # PERCENT-COMPLETE
//!
//! The `PERCENT-COMPLETE` property: how far a to-do has got, 0 to 100 (RFC 5545
//! 3.8.1.8).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `PERCENT-COMPLETE` property marker.
#[allow(non_camel_case_types)]
pub struct PERCENT_COMPLETE;

impl IcalPropSpec for PERCENT_COMPLETE {
    const KIND: IcalPropKind = IcalPropKind::PercentComplete;

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
