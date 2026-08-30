//! # VTODO
//!
//! The `VTODO` component: a to-do item (RFC 5545 3.6.2).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `VTODO` component marker.
pub struct VTODO;

impl IcalComponentSpec for VTODO {
    const KIND: IcalComponentKind = IcalComponentKind::VTodo;

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
