//! # CONFERENCE
//!
//! The `CONFERENCE` property: how to join the component's conference, by URI
//! (RFC 7986 5.11).

use crate::{
    param::IcalParamKind,
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `CONFERENCE` property marker.
pub struct CONFERENCE;

impl IcalPropSpec for CONFERENCE {
    const KIND: IcalPropKind = IcalPropKind::Conference;

    fn allowed_versions() -> &'static [IcalVersion] {
        &[IcalVersion::V2_0]
    }

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::Uri]
    }

    /// What the access offers and a label for it (RFC 7986 6.3, 6.4), in a
    /// language.
    fn allowed_params(_version: IcalVersion) -> &'static [IcalParamKind] {
        &[
            IcalParamKind::Value,
            IcalParamKind::Feature,
            IcalParamKind::Label,
            IcalParamKind::Language,
        ]
    }
}
