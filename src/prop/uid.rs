//! # UID
//!
//! The `UID` property: the persistent, globally unique identifier of the
//! component (RFC 5545 3.8.4.7).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `UID` property marker.
pub struct UID;

impl IcalPropSpec for UID {
    const KIND: IcalPropKind = IcalPropKind::Uid;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
