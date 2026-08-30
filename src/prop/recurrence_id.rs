//! # RECURRENCE-ID
//!
//! The `RECURRENCE-ID` property: which instance of a series this component
//! overrides (RFC 5545 3.8.4.4).

use crate::{
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

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::DateTime]
    }
}
