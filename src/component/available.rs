//! # AVAILABLE
//!
//! The `AVAILABLE` component: one available period of a `VAVAILABILITY` (RFC
//! 7953 3.1).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `AVAILABLE` component marker.
pub struct AVAILABLE;

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
