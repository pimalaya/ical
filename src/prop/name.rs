//! # NAME
//!
//! The `NAME` property: the display name of the calendar (RFC 7986 5.1).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `NAME` property marker.
pub struct NAME;

impl IcalPropSpec for NAME {
    const KIND: IcalPropKind = IcalPropKind::Name;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
