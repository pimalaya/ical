//! # VEVENT component lens
//!
//! The `VEVENT` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{IcalComponentLens, IcalComponentSpec},
};

/// The `VEVENT` component lens.
#[allow(non_camel_case_types)]
pub struct VEVENT;

impl IcalComponentLens for VEVENT {}

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
