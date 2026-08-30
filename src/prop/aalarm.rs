//! # AALARM
//!
//! The `AALARM` property: the audio alarm a vCalendar 1.0 component carries,
//! run time first, kept as raw text (vCalendar 1.0).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `AALARM` property marker.
pub struct AALARM;

impl IcalPropSpec for AALARM {
    const KIND: IcalPropKind = IcalPropKind::AAlarm;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }
}
