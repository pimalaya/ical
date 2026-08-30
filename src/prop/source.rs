//! # SOURCE
//!
//! The `SOURCE` property: where the calendar itself can be refetched (RFC 7986
//! 5.8).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `SOURCE` property marker.
pub struct SOURCE;

impl IcalPropSpec for SOURCE {
    const KIND: IcalPropKind = IcalPropKind::Source;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
