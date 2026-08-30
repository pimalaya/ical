//! # VTIMEZONE
//!
//! The `VTIMEZONE` component: a time zone, carrying its own transition rules
//! (RFC 5545 3.6.5).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `VTIMEZONE` component marker.
pub struct VTIMEZONE;

impl IcalComponentSpec for VTIMEZONE {
    const KIND: IcalComponentKind = IcalComponentKind::VTimezone;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::Standard, IcalComponentKind::Daylight]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::TzId]
    }
}
