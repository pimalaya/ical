//! # PALARM
//!
//! The `PALARM` property: the procedure alarm a vCalendar 1.0 component
//! carries, kept as raw text (vCalendar 1.0).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `PALARM` property marker.
pub struct PALARM;

impl IcalPropSpec for PALARM {
    const KIND: IcalPropKind = IcalPropKind::PAlarm;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V1_0]
    }
}
