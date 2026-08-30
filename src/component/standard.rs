//! # STANDARD
//!
//! The `STANDARD` component: the standard-time observance of a `VTIMEZONE` (RFC
//! 5545 3.6.5).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `STANDARD` component marker.
pub struct STANDARD;

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
