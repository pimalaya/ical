//! # COMPLETED
//!
//! The `COMPLETED` property: when a to-do was completed (RFC 5545 3.8.2.1).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `COMPLETED` property marker.
pub struct COMPLETED;

impl IcalPropSpec for COMPLETED {
    const KIND: IcalPropKind = IcalPropKind::Completed;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
