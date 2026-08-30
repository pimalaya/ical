//! # VEVENT
//!
//! The `VEVENT` component: a scheduled event (RFC 5545 3.6.1).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `VEVENT` component marker.
pub struct VEVENT;

impl IcalComponentSpec for VEVENT {
    const KIND: IcalComponentKind = IcalComponentKind::VEvent;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[
            IcalComponentKind::VAlarm,
            IcalComponentKind::Participant,
            IcalComponentKind::VLocation,
            IcalComponentKind::VResource,
        ]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::Uid, IcalPropKind::DtStamp]
    }
}
