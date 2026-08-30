//! # DTSTART
//!
//! The `DTSTART` property: when the component starts, and the clock a
//! recurrence expands on (RFC 5545 3.8.2.4).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `DTSTART` property marker.
pub struct DTSTART;

impl IcalPropSpec for DTSTART {
    const KIND: IcalPropKind = IcalPropKind::DtStart;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
