//! # TZNAME
//!
//! The `TZNAME` property: the customary name of a time-zone observance, such as
//! `EST` (RFC 5545 3.8.3.2).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `TZNAME` property marker.
pub struct TZNAME;

impl IcalPropSpec for TZNAME {
    const KIND: IcalPropKind = IcalPropKind::TzName;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
