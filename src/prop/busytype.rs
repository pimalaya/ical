//! # BUSYTYPE
//!
//! The `BUSYTYPE` property: the kind of busy time a `VAVAILABILITY` describes
//! (RFC 7953 3.2).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `BUSYTYPE` property marker.
pub struct BUSYTYPE;

impl IcalPropSpec for BUSYTYPE {
    const KIND: IcalPropKind = IcalPropKind::BusyType;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
