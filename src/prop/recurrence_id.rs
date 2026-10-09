//! # RECURRENCE-ID
//!
//! The `RECURRENCE-ID` property: which instance of a series this component
//! overrides (RFC 5545 3.8.4.4).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `RECURRENCE-ID` property marker.
#[allow(non_camel_case_types)]
pub struct RECURRENCE_ID;

impl IcalPropSpec for RECURRENCE_ID {
    const KIND: IcalPropKind = IcalPropKind::RecurrenceId;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    /// An instance of a whole-day series is a `DATE`, as its `DTSTART` is.
    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime, IcalValueKind::Date]
    }

    /// A local time names its zone with `TZID`, and `RANGE=THISANDFUTURE`
    /// stretches the override over every later instance.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::TzId,
            IcalParamKind::Range,
        ]
    }
}
