//! # IMAGE
//!
//! The `IMAGE` property: an image for the calendar or component, by URI or
//! inline (RFC 7986 5.10).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `IMAGE` property marker.
pub struct IMAGE;

impl IcalPropSpec for IMAGE {
    const KIND: IcalPropKind = IcalPropKind::Image;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
