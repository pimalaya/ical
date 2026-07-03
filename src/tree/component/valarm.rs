//! # VALARM component lens
//!
//! The `VALARM` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{IcalComponentLens, IcalComponentSpec},
};

/// The `VALARM` component lens.
#[allow(non_camel_case_types)]
pub struct VALARM;

impl IcalComponentLens for VALARM {}

impl IcalComponentSpec for VALARM {
    const KIND: IcalComponentKind = IcalComponentKind::VAlarm;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::VLocation]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::Action, IcalPropKind::Trigger]
    }
}
