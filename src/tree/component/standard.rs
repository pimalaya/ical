//! # STANDARD component lens
//!
//! The `STANDARD` component lens.

use crate::{
    component::IcalComponentKind,
    prop::IcalPropKind,
    tree::component::{lens::IcalComponentLens, spec::IcalComponentSpec},
};

/// The `STANDARD` component lens.
#[allow(non_camel_case_types)]
pub struct STANDARD;

impl IcalComponentLens for STANDARD {}

impl IcalComponentSpec for STANDARD {
    const KIND: IcalComponentKind = IcalComponentKind::Standard;

    fn required_props() -> &'static [IcalPropKind] {
        &[
            IcalPropKind::DtStart,
            IcalPropKind::TzOffsetFrom,
            IcalPropKind::TzOffsetTo,
        ]
    }
}
