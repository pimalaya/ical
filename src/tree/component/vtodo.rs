//! # VTODO component lens
//!
//! The `VTODO` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{lens::IcalComponentLens, spec::IcalComponentSpec},
};

/// The `VTODO` component lens.
#[allow(non_camel_case_types)]
pub struct VTODO;

impl IcalComponentLens for VTODO {}

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
