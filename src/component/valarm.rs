//! # VALARM
//!
//! The `VALARM` component: an alarm on the component holding it (RFC 5545
//! 3.6.6).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `VALARM` component marker.
pub struct VALARM;

impl IcalComponentSpec for VALARM {
    const KIND: IcalComponentKind = IcalComponentKind::VAlarm;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::VLocation]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::Action, IcalPropKind::Trigger]
    }
}
