//! # VRESOURCE
//!
//! The `VRESOURCE` component: a resource the component holding it needs (RFC
//! 9073 7.3).

use crate::component::{IcalComponentKind, spec::IcalComponentSpec};

/// The `VRESOURCE` component marker.
pub struct VRESOURCE;

impl IcalComponentSpec for VRESOURCE {
    const KIND: IcalComponentKind = IcalComponentKind::VResource;
}
