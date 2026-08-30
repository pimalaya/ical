//! # CONFERENCE
//!
//! The `CONFERENCE` property: how to join the component's conference, by URI
//! (RFC 7986 5.11).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `CONFERENCE` property marker.
pub struct CONFERENCE;

impl IcalPropSpec for CONFERENCE {
    const KIND: IcalPropKind = IcalPropKind::Conference;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
