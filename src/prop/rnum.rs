//! # RNUM
//!
//! The `RNUM` property: how many occurrences a vCalendar 1.0 recurrence rule
//! generates (vCalendar 1.0).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `RNUM` property marker.
pub struct RNUM;

impl IcalPropSpec for RNUM {
    const KIND: IcalPropKind = IcalPropKind::RNum;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Integer]
    }
}
