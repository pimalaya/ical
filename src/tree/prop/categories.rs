//! # CATEGORIES lens
//!
//! The `CATEGORIES` property lens.

use crate::{
    prop::IcalPropKind,
    tree::{
        line::IcalLine,
        prop::{IcalPropLens, IcalPropSpec},
        value::IcalValueCursor,
    },
    value::IcalValueKind,
    value::text::IcalTextList,
    version::IcalVersion,
};

/// The `CATEGORIES` property lens.
#[allow(non_camel_case_types)]
pub struct CATEGORIES;

impl IcalPropLens for CATEGORIES {
    type Target<'v> = IcalTextList<'v>;

    type Cursor<'c, 'a>
        = IcalValueCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut IcalLine<'a>) -> IcalValueCursor<'c, 'a> {
        IcalValueCursor { line }
    }
}

impl IcalPropSpec for CATEGORIES {
    const KIND: IcalPropKind = IcalPropKind::Categories;

    fn allowed_values(_version: IcalVersion) -> &'static [IcalValueKind] {
        &[IcalValueKind::TextList]
    }
}
