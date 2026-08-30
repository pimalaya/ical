//! # CATEGORIES
//!
//! The `CATEGORIES` property: the categories the component belongs to (RFC 5545
//! 3.8.1.2).

use crate::{
    prop::{IcalPropKind, spec::IcalPropSpec},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The `CATEGORIES` property marker.
pub struct CATEGORIES;

impl IcalPropSpec for CATEGORIES {
    const KIND: IcalPropKind = IcalPropKind::Categories;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::TextList]
    }

    /// A list whatever `VALUE` declares: `TEXT` describes each item, not the
    /// value as a whole.
    fn value(_version: IcalVersion, _declared: Option<IcalValueKind>) -> IcalValueKind {
        IcalValueKind::TextList
    }
}
