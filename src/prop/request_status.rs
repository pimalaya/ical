//! # REQUEST-STATUS
//!
//! The `REQUEST-STATUS` property: the outcome of a scheduling request, a code,
//! a message and optional data (RFC 5545 3.8.8.3).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `REQUEST-STATUS` property marker.
#[allow(non_camel_case_types)]
pub struct REQUEST_STATUS;

impl IcalPropSpec for REQUEST_STATUS {
    const KIND: IcalPropKind = IcalPropKind::RequestStatus;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::RequestStatus]
    }
}
