//! # VLOCATION component lens
//!
//! The `VLOCATION` component lens.

use crate::{
    component::IcalComponentKind,
    tree::component::{IcalComponentLens, IcalComponentSpec},
};

/// The `VLOCATION` component lens.
#[allow(non_camel_case_types)]
pub struct VLOCATION;

impl IcalComponentLens for VLOCATION {}

impl IcalComponentSpec for VLOCATION {
    const KIND: IcalComponentKind = IcalComponentKind::VLocation;
}
