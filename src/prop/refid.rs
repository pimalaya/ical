//! # REFID
//!
//! The `REFID` property: the reference identifier a group of related components
//! shares (RFC 9253 6.2).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `REFID` property marker.
pub struct REFID;

impl IcalPropSpec for REFID {
    const KIND: IcalPropKind = IcalPropKind::Refid;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
