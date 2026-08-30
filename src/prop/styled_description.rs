//! # STYLED-DESCRIPTION
//!
//! The `STYLED-DESCRIPTION` property: the description in a richer text format
//! than `DESCRIPTION` (RFC 9073 6.5).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    version::IcalVersion,
};

/// The `STYLED-DESCRIPTION` property marker.
#[allow(non_camel_case_types)]
pub struct STYLED_DESCRIPTION;

impl IcalPropSpec for STYLED_DESCRIPTION {
    const KIND: IcalPropKind = IcalPropKind::StyledDescription;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }
}
