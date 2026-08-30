//! # STRUCTURED-DATA
//!
//! The `STRUCTURED-DATA` property: machine-readable data about the component,
//! inline or by URI (RFC 9073 6.6).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `STRUCTURED-DATA` property marker.
#[allow(non_camel_case_types)]
pub struct STRUCTURED_DATA;

impl IcalPropSpec for STRUCTURED_DATA {
    const KIND: IcalPropKind = IcalPropKind::StructuredData;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
