//! # ATTACH
//!
//! The `ATTACH` property: a document associated with the component, a URI or an
//! inline `BASE64` body (RFC 5545 3.8.1.1).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `ATTACH` property marker.
pub struct ATTACH;

impl IcalPropSpec for ATTACH {
    const KIND: IcalPropKind = IcalPropKind::Attach;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
