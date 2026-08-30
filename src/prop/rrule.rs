//! # RRULE
//!
//! The `RRULE` property: the rule generating the component's recurrence set
//! (RFC 5545 3.8.5.3).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `RRULE` property marker.
pub struct RRULE;

impl IcalPropSpec for RRULE {
    const KIND: IcalPropKind = IcalPropKind::RRule;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Recur]
    }
}
