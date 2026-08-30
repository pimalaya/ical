//! # DALARM
//!
//! The `DALARM` property: the display alarm a vCalendar 1.0 component carries,
//! kept as raw text (vCalendar 1.0).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `DALARM` property marker.
pub struct DALARM;

impl IcalPropSpec for DALARM {
    const KIND: IcalPropKind = IcalPropKind::DAlarm;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }
}
