//! # VLOCATION
//!
//! The `VLOCATION` component: a location of the component holding it (RFC 9073
//! 7.2).

use crate::component::{IcalComponentKind, spec::IcalComponentSpec};

/// The `VLOCATION` component marker.
pub struct VLOCATION;

impl IcalComponentSpec for VLOCATION {
    const KIND: IcalComponentKind = IcalComponentKind::VLocation;
}
