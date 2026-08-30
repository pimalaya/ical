//! # ACTION
//!
//! The `ACTION` property: what an alarm does when it fires: `AUDIO`, `DISPLAY`
//! or `EMAIL` (RFC 5545 3.8.6.1).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `ACTION` property marker.
pub struct ACTION;

impl IcalPropSpec for ACTION {
    const KIND: IcalPropKind = IcalPropKind::Action;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
