//! # VFREEBUSY
//!
//! The `VFREEBUSY` component: a free/busy reply or publication (RFC 5545
//! 3.6.4).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `VFREEBUSY` component marker.
pub struct VFREEBUSY;

impl IcalComponentSpec for VFREEBUSY {
    const KIND: IcalComponentKind = IcalComponentKind::VFreeBusy;

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::Uid, IcalPropKind::DtStamp]
    }
}
