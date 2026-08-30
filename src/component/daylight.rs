//! # DAYLIGHT
//!
//! The `DAYLIGHT` component: the daylight-saving observance of a `VTIMEZONE`
//! (RFC 5545 3.6.5).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `DAYLIGHT` component marker.
pub struct DAYLIGHT;

impl IcalComponentSpec for DAYLIGHT {
    const KIND: IcalComponentKind = IcalComponentKind::Daylight;

    fn required_props() -> &'static [IcalPropKind] {
        &[
            IcalPropKind::DtStart,
            IcalPropKind::TzOffsetFrom,
            IcalPropKind::TzOffsetTo,
        ]
    }
}
