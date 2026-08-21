//! # VCALENDAR component lens
//!
//! The `VCALENDAR` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{lens::IcalComponentLens, spec::IcalComponentSpec},
};

/// The `VCALENDAR` component lens.
#[allow(non_camel_case_types)]
pub struct VCALENDAR;

impl IcalComponentLens for VCALENDAR {}

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
