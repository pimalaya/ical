//! # PRODID
//!
//! The `PRODID` property: the product that wrote the calendar (RFC 5545 3.7.3).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `PRODID` property marker.
pub struct PRODID;

impl IcalPropSpec for PRODID {
    const KIND: IcalPropKind = IcalPropKind::ProdId;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
