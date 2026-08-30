//! # MALARM
//!
//! The `MALARM` property: the mail alarm a vCalendar 1.0 component carries,
//! kept as raw text (vCalendar 1.0).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `MALARM` property marker.
pub struct MALARM;

impl IcalPropSpec for MALARM {
    const KIND: IcalPropKind = IcalPropKind::MAlarm;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }
}
