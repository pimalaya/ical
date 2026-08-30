//! # SUMMARY
//!
//! The `SUMMARY` property: the short summary or subject of the component (RFC
//! 5545 3.8.1.12).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `SUMMARY` property marker.
pub struct SUMMARY;

impl IcalPropSpec for SUMMARY {
    const KIND: IcalPropKind = IcalPropKind::Summary;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
