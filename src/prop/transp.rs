//! # TRANSP
//!
//! The `TRANSP` property: whether the component blocks free/busy time: `OPAQUE`
//! or `TRANSPARENT` (RFC 5545 3.8.2.7).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `TRANSP` property marker.
pub struct TRANSP;

impl IcalPropSpec for TRANSP {
    const KIND: IcalPropKind = IcalPropKind::Transp;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
