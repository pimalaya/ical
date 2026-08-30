//! # RESOURCE-TYPE
//!
//! The `RESOURCE-TYPE` property: what kind of thing a `VRESOURCE` is (RFC 9073
//! 6.3).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `RESOURCE-TYPE` property marker.
#[allow(non_camel_case_types)]
pub struct RESOURCE_TYPE;

impl IcalPropSpec for RESOURCE_TYPE {
    const KIND: IcalPropKind = IcalPropKind::ResourceType;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
