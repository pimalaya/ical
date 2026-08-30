//! # METHOD
//!
//! The `METHOD` property: the iTIP method the calendar carries: `REQUEST`,
//! `REPLY`, `CANCEL`, ... (RFC 5545 3.7.2).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `METHOD` property marker.
pub struct METHOD;

impl IcalPropSpec for METHOD {
    const KIND: IcalPropKind = IcalPropKind::Method;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
