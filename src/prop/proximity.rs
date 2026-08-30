//! # PROXIMITY
//!
//! The `PROXIMITY` property: the location trigger of a `VALARM`: `ARRIVE`,
//! `DEPART`, ... (RFC 9074 8.1).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `PROXIMITY` property marker.
pub struct PROXIMITY;

impl IcalPropSpec for PROXIMITY {
    const KIND: IcalPropKind = IcalPropKind::Proximity;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
