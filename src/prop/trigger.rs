//! # TRIGGER
//!
//! The `TRIGGER` property: when an alarm fires, relative to the component or at
//! an absolute time (RFC 5545 3.8.6.3).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `TRIGGER` property marker.
pub struct TRIGGER;

impl IcalPropSpec for TRIGGER {
    const KIND: IcalPropKind = IcalPropKind::Trigger;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    /// An absolute trigger is a `DATE-TIME` beside the default `DURATION`.
    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Duration, IcalValueKind::DateTime]
    }

    /// A relative trigger counts from the start or the end, by `RELATED`.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[IcalParamKind::Value, IcalParamKind::Related]
    }
}
