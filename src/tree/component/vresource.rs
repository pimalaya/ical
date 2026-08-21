//! # VRESOURCE component lens
//!
//! The `VRESOURCE` component lens.

use crate::{
    component::IcalComponentKind,
    tree::component::{lens::IcalComponentLens, spec::IcalComponentSpec},
};

/// The `VRESOURCE` component lens.
#[allow(non_camel_case_types)]
pub struct VRESOURCE;

impl IcalComponentLens for VRESOURCE {}

impl IcalComponentSpec for VRESOURCE {
    const KIND: IcalComponentKind = IcalComponentKind::VResource;
}
