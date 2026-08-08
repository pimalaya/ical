//! # VAVAILABILITY component lens
//!
//! The `VAVAILABILITY` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{IcalComponentLens, IcalComponentSpec},
};

/// The `VAVAILABILITY` component lens.
#[allow(non_camel_case_types)]
pub struct VAVAILABILITY;

impl IcalComponentLens for VAVAILABILITY {}

impl IcalComponentSpec for VAVAILABILITY {
    const KIND: IcalComponentKind = IcalComponentKind::VAvailability;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::Available]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::DtStamp, IcalPropKind::Uid]
    }
}
