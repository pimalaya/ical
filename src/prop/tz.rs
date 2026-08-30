//! # TZ
//!
//! The `TZ` property: the UTC offset a whole vCalendar 1.0 calendar is written
//! in (vCalendar 1.0).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `TZ` property marker.
pub struct TZ;

impl IcalPropSpec for TZ {
    const KIND: IcalPropKind = IcalPropKind::Tz;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::UtcOffset]
    }
}
