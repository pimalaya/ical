//! # VCALENDAR
//!
//! The `VCALENDAR` component: the calendar envelope every other component sits
//! in (RFC 5545 3.4).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `VCALENDAR` component marker.
pub struct VCALENDAR;

impl IcalComponentSpec for VCALENDAR {
    const KIND: IcalComponentKind = IcalComponentKind::VCalendar;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[
            IcalComponentKind::VEvent,
            IcalComponentKind::VTodo,
            IcalComponentKind::VJournal,
            IcalComponentKind::VFreeBusy,
            IcalComponentKind::VTimezone,
            IcalComponentKind::VAvailability,
        ]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::ProdId]
    }
}
