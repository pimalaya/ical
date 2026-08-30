//! # CALSCALE
//!
//! The `CALSCALE` property: the calendar scale the dates are read in,
//! `GREGORIAN` in practice (RFC 5545 3.7.1).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `CALSCALE` property marker.
pub struct CALSCALE;

impl IcalPropSpec for CALSCALE {
    const KIND: IcalPropKind = IcalPropKind::CalScale;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
