//! # DAYLIGHT component lens
//!
//! The `DAYLIGHT` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{IcalComponentLens, IcalComponentSpec},
};

/// The `DAYLIGHT` component lens.
#[allow(non_camel_case_types)]
pub struct DAYLIGHT;

impl IcalComponentLens for DAYLIGHT {}

impl IcalComponentSpec for DAYLIGHT {
    const KIND: IcalComponentKind = IcalComponentKind::Daylight;

    fn required_props() -> &'static [IcalPropKind] {
        &[
            IcalPropKind::DtStart,
            IcalPropKind::TzOffsetFrom,
            IcalPropKind::TzOffsetTo,
        ]
    }
}
