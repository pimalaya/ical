//! # LOCATION-TYPE
//!
//! The `LOCATION-TYPE` property: what kind of place a `VLOCATION` is (RFC 9073
//! 6.1).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `LOCATION-TYPE` property marker.
#[allow(non_camel_case_types)]
pub struct LOCATION_TYPE;

impl IcalPropSpec for LOCATION_TYPE {
    const KIND: IcalPropKind = IcalPropKind::LocationType;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
