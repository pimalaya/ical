//! # GEO
//!
//! The `GEO` property: the global position of the component, latitude and
//! longitude (RFC 5545 3.8.1.6).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `GEO` property marker.
pub struct GEO;

impl IcalPropSpec for GEO {
    const KIND: IcalPropKind = IcalPropKind::Geo;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Geo]
    }
}
