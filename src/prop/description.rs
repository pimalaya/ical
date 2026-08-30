//! # DESCRIPTION
//!
//! The `DESCRIPTION` property: the long description of the component (RFC 5545
//! 3.8.1.5).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `DESCRIPTION` property marker.
pub struct DESCRIPTION;

impl IcalPropSpec for DESCRIPTION {
    const KIND: IcalPropKind = IcalPropKind::Description;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
