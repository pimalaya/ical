//! # FREEBUSY
//!
//! The `FREEBUSY` property: one free or busy period of a `VFREEBUSY` (RFC 5545
//! 3.8.2.6).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `FREEBUSY` property marker.
pub struct FREEBUSY;

impl IcalPropSpec for FREEBUSY {
    const KIND: IcalPropKind = IcalPropKind::FreeBusy;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Period]
    }
}
