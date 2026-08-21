//! # VFREEBUSY component lens
//!
//! The `VFREEBUSY` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{lens::IcalComponentLens, spec::IcalComponentSpec},
};

/// The `VFREEBUSY` component lens.
#[allow(non_camel_case_types)]
pub struct VFREEBUSY;

impl IcalComponentLens for VFREEBUSY {}

impl IcalComponentSpec for VFREEBUSY {
    const KIND: IcalComponentKind = IcalComponentKind::VFreeBusy;

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::Uid, IcalPropKind::DtStamp]
    }
}
