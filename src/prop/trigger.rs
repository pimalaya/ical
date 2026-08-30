//! # TRIGGER
//!
//! The `TRIGGER` property: when an alarm fires, relative to the component or at
//! an absolute time (RFC 5545 3.8.6.3).

use crate::{
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

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Duration]
    }
}
