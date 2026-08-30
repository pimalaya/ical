//! # CONTACT
//!
//! The `CONTACT` property: who to contact about the component (RFC 5545
//! 3.8.4.2).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `CONTACT` property marker.
pub struct CONTACT;

impl IcalPropSpec for CONTACT {
    const KIND: IcalPropKind = IcalPropKind::Contact;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
