//! # STATUS
//!
//! The `STATUS` property: the overall status: `TENTATIVE`, `CONFIRMED`,
//! `CANCELLED`, ... (RFC 5545 3.8.1.11).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `STATUS` property marker.
pub struct STATUS;

impl IcalPropSpec for STATUS {
    const KIND: IcalPropKind = IcalPropKind::Status;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
