//! # AVAILABLE component lens
//!
//! The `AVAILABLE` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{IcalComponentLens, IcalComponentSpec},
};

/// The `AVAILABLE` component lens.
#[allow(non_camel_case_types)]
pub struct AVAILABLE;

impl IcalComponentLens for AVAILABLE {}

impl IcalComponentSpec for AVAILABLE {
    const KIND: IcalComponentKind = IcalComponentKind::Available;

    fn required_props() -> &'static [IcalPropKind] {
        &[
            IcalPropKind::DtStamp,
            IcalPropKind::DtStart,
            IcalPropKind::Uid,
        ]
    }
}
