//! # SEQUENCE
//!
//! The `SEQUENCE` property: the revision number of the component, raised on
//! every significant change (RFC 5545 3.8.7.4).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `SEQUENCE` property marker.
pub struct SEQUENCE;

impl IcalPropSpec for SEQUENCE {
    const KIND: IcalPropKind = IcalPropKind::Sequence;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Integer]
    }
}
