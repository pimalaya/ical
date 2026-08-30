//! # CALENDAR-ADDRESS
//!
//! The `CALENDAR-ADDRESS` property: the calendar user address of a
//! `PARTICIPANT` (RFC 9073 6.4).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `CALENDAR-ADDRESS` property marker.
#[allow(non_camel_case_types)]
pub struct CALENDAR_ADDRESS;

impl IcalPropSpec for CALENDAR_ADDRESS {
    const KIND: IcalPropKind = IcalPropKind::CalendarAddress;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::CalAddress]
    }
}
