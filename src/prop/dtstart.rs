//! # DTSTART
//!
//! The `DTSTART` property: when the component starts, and the clock a
//! recurrence expands on (RFC 5545 3.8.2.4).

use crate::{
    param::IcalParamKind,
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

    /// A whole day is a `DATE` beside the default `DATE-TIME`.
    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime, IcalValueKind::Date]
    }

    /// A local time names its zone with `TZID`.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[IcalParamKind::Value, IcalParamKind::TzId]
    }
}
