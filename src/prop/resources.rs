//! # RESOURCES
//!
//! The `RESOURCES` property: the resources the component needs (RFC 5545
//! 3.8.1.10).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `RESOURCES` property marker.
pub struct RESOURCES;

impl IcalPropSpec for RESOURCES {
    const KIND: IcalPropKind = IcalPropKind::Resources;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::TextList]
    }

    /// A list whatever `VALUE` declares: `TEXT` describes each item, not the
    /// value as a whole.
    fn value(_version: IcalVersion, _declared: Option<IcalValueKind>) -> IcalValueKind {
        IcalValueKind::TextList
    }
}
