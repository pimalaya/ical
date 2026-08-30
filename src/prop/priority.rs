//! # PRIORITY
//!
//! The `PRIORITY` property: the relative priority of the component, 0
//! (undefined) to 9 (lowest) (RFC 5545 3.8.1.9).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `PRIORITY` property marker.
pub struct PRIORITY;

impl IcalPropSpec for PRIORITY {
    const KIND: IcalPropKind = IcalPropKind::Priority;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Integer]
    }
}
