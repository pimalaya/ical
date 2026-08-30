//! # COLOR
//!
//! The `COLOR` property: the colour a client should show the calendar or
//! component in, a CSS3 name (RFC 7986 5.9).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `COLOR` property marker.
pub struct COLOR;

impl IcalPropSpec for COLOR {
    const KIND: IcalPropKind = IcalPropKind::Color;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }
}
