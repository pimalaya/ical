//! # CLASS
//!
//! The `CLASS` property: the access classification: `PUBLIC`, `PRIVATE` or
//! `CONFIDENTIAL` (RFC 5545 3.8.1.3).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `CLASS` property marker.
pub struct CLASS;

impl IcalPropSpec for CLASS {
    const KIND: IcalPropKind = IcalPropKind::Class;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
