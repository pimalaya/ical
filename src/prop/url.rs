//! # URL
//!
//! The `URL` property: a URL for the component (RFC 5545 3.8.4.6).

use crate::{
    prop::{IcalPropKind, cardinality::IcalPropCardinality, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `URL` property marker.
pub struct URL;

impl IcalPropSpec for URL {
    const KIND: IcalPropKind = IcalPropKind::Url;

    fn cardinality(_version: IcalVersion) -> IcalPropCardinality {
        IcalPropCardinality::AtMostOne
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }
}
