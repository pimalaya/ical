//! # VAVAILABILITY
//!
//! The `VAVAILABILITY` component: the periods a calendar user is available in
//! (RFC 7953 3.1).

use crate::{
    component::{IcalComponentKind, spec::IcalComponentSpec},
    prop::IcalPropKind,
};

/// The `VAVAILABILITY` component marker.
pub struct VAVAILABILITY;

impl IcalComponentSpec for VAVAILABILITY {
    const KIND: IcalComponentKind = IcalComponentKind::VAvailability;

    fn allowed_children() -> &'static [IcalComponentKind] {
        &[IcalComponentKind::Available]
    }

    fn required_props() -> &'static [IcalPropKind] {
        &[IcalPropKind::DtStamp, IcalPropKind::Uid]
    }
}
