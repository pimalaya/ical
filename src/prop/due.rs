//! # DUE
//!
//! The `DUE` property: when a to-do is due (RFC 5545 3.8.2.3).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `DUE` property marker.
pub struct DUE;

impl IcalPropSpec for DUE {
    const KIND: IcalPropKind = IcalPropKind::Due;

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
