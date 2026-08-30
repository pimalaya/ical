//! # LOCATION
//!
//! The `LOCATION` property: where the component takes place (RFC 5545 3.8.1.7).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `LOCATION` property marker.
pub struct LOCATION;

impl IcalPropSpec for LOCATION {
    const KIND: IcalPropKind = IcalPropKind::Location;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
