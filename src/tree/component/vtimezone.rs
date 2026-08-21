//! # VTIMEZONE component lens
//!
//! The `VTIMEZONE` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{lens::IcalComponentLens, spec::IcalComponentSpec},
};

/// The `VTIMEZONE` component lens.
#[allow(non_camel_case_types)]
pub struct VTIMEZONE;

impl IcalComponentLens for VTIMEZONE {}

impl IcalComponentSpec for VTIMEZONE {
    const KIND: IcalComponentKind = IcalComponentKind::VTimezone;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::Standard, IcalComponentKind::Daylight]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::TzId]
    }
}
